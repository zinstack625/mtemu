#include "mono/metadata/appdomain.h"
#include "mono/metadata/class.h"
#include "mono/metadata/image.h"
#include "mono/metadata/object-forward.h"
#include "mono/metadata/object.h"
#include <mono/jit/jit.h>
#include <mono/metadata/assembly.h>
#include <mono/metadata/debug-helpers.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
  int isOffset;
  int number_;
  int *words;
  size_t words_len;
} Command;

MonoObject *instance_emulator(MonoDomain *dom, MonoImage *im) {
  MonoClass *PortExtender = mono_class_from_name(im, "mtemu", "PortExtender");
  MonoObject *port_extender = mono_object_new(dom, PortExtender);
  /* TODO: currently broken for whatever reason, will dig deeper later */
  /* MonoMethodDesc* PortExtenderCtorDesc =
   * mono_method_desc_new("mtemu.PortExtender:.ctor()", 1); */
  /* MonoMethod* PortExtenderCtor =
   * mono_method_desc_search_in_class(PortExtenderCtorDesc, PortExtender); */
  /* mono_runtime_invoke(PortExtenderCtor, port_extender, NULL, NULL); */
  MonoClass *Emulator = mono_class_from_name(im, "mtemu", "Emulator");
  MonoObject *emulator = mono_object_new(dom, Emulator);
  // MonoMethodDesc* EmulatorCtorDesc =
  // mono_method_desc_new("mtemu.Emulator:.ctor(PortExtender)", 1);
  MonoMethodDesc *EmulatorCtorDesc =
      mono_method_desc_new("mtemu.Emulator:.ctor()", 1);
  MonoMethod *EmulatorCtor =
      mono_method_desc_search_in_class(EmulatorCtorDesc, Emulator);
  /* void* args[1]; */
  /* args[0] = &port_extender; */
  /* mono_runtime_invoke(EmulatorCtor, emulator, args, NULL); */
  mono_runtime_invoke(EmulatorCtor, emulator, NULL, NULL);
  return emulator;
}

void emulator_load_file(MonoDomain *dom, MonoObject *emul,
                        const char *filename) {
  MonoClass *emul_class = mono_object_get_class(emul);
  MonoMethodDesc *emu_RawFile_desc =
      mono_method_desc_new("mtemu.Emulator:OpenRaw(byte[])", 1);
  MonoMethod *emu_OpenFile =
      mono_method_desc_search_in_class(emu_RawFile_desc, emul_class);
  MonoMethodDesc *emu_GetLen_desc =
      mono_method_desc_new("mtemu.Emulator:GetLen(byte[])", 1);
  MonoMethod *emu_GetLen =
      mono_method_desc_search_in_class(emu_GetLen_desc, emul_class);
  FILE *mtemu_prog_file = fopen(filename, "rb");
  fseek(mtemu_prog_file, 0, SEEK_END);
  size_t fsize = ftell(mtemu_prog_file);
  fseek(mtemu_prog_file, 0, SEEK_SET);
  uint8_t *buf = malloc(fsize * sizeof(uint8_t));
  fread(buf, fsize, 1, mtemu_prog_file);
  fclose(mtemu_prog_file);
  MonoArray *file_bytes = mono_array_new(dom, mono_get_byte_class(), fsize);
  for (size_t i = 0; i < fsize; ++i) {
    mono_array_set(file_bytes, uint8_t, i, buf[i]);
  }
  void *args[1];
  args[0] = file_bytes;
  int *arrLen =
      mono_object_unbox(mono_runtime_invoke(emu_GetLen, emul, args, NULL));
  printf("%i\n", *arrLen);
  mono_runtime_invoke(emu_OpenFile, emul, args, NULL);
  // free(buf);
}

MonoObject *emulator_get_command(MonoDomain *dom, MonoImage *im,
                                 MonoObject *emul, size_t index) {
  MonoClass *emulator_class = mono_object_get_class(emul);
  MonoMethodDesc *emu_GetCommand_desc =
      mono_method_desc_new("mtemu.Emulator:GetCommand(int)", 1);
  MonoMethod *emu_GetCommand =
      mono_method_desc_search_in_class(emu_GetCommand_desc, emulator_class);
  void *args[1];
  args[0] = &index;
  MonoObject *res = mono_runtime_invoke(emu_GetCommand, emul, args, NULL);
  return res;
}

Command command_through_interop(MonoDomain *dom, MonoImage *im,
                                MonoObject *cmd_obj) {
  MonoClass *cmd_Command = mono_class_from_name(im, "mtemu", "Command");
  MonoClassField *isOffset =
      mono_class_get_field_from_name(cmd_Command, "isOffset");
  MonoClassField *number_ =
      mono_class_get_field_from_name(cmd_Command, "number_");
  MonoClassField *words_ =
      mono_class_get_field_from_name(cmd_Command, "words_");
  Command cmd;
  mono_field_get_value(cmd_obj, isOffset, &cmd.isOffset);
  mono_field_get_value(cmd_obj, number_, &cmd.number_);
  MonoArray *words_arr =
      (MonoArray *)mono_field_get_value_object(dom, words_, cmd_obj);
  cmd.words_len = mono_array_length(words_arr);
  cmd.words = malloc(cmd.words_len);
  for (size_t i = 0; i < cmd.words_len; ++i) {
    cmd.words[i] = mono_array_get(words_arr, int, i);
  }
  return cmd;
}

char *command_get_name(MonoDomain *dom, MonoImage *im, MonoObject *cmd_obj) {
  MonoClass *command_class = mono_object_get_class(cmd_obj);
  MonoMethodDesc *cmd_GetName_desc =
      mono_method_desc_new("mtemu.Command:GetName", 1);
  MonoMethod *cmd_GetName =
      mono_method_desc_search_in_class(cmd_GetName_desc, command_class);
  MonoString *res_str =
      (MonoString *)mono_runtime_invoke(cmd_GetName, cmd_obj, NULL, NULL);
  return mono_string_to_utf8(res_str);
}

int main() {
  MonoDomain *exec_dom = mono_jit_init("mtemu");
  MonoAssembly *mtemu_emu = mono_domain_assembly_open(
      exec_dom, "src/emulator/implementation/engine.dll");
  if (!mtemu_emu) {
    exit(1);
  }
  MonoImage *image = mono_assembly_get_image(mtemu_emu);
  MonoObject *emulator = instance_emulator(exec_dom, image);
  emulator_load_file(exec_dom, emulator, "/home/anton/mtemu_prog.mte");

  MonoObject *command = emulator_get_command(exec_dom, image, emulator, 0);
  Command native_command = command_through_interop(exec_dom, image, command);
  printf("isOffset:\t%i\nnumber_:\t%i\n", native_command.isOffset,
         native_command.number_);
  for (size_t i = 0; i < native_command.words_len; ++i) {
    printf("words_[%li]:\t%i\n", i, native_command.words[i]);
  }

  const char *command_name = command_get_name(exec_dom, image, command);
  printf("%s\n", command_name);

  mono_jit_cleanup(exec_dom);
}
