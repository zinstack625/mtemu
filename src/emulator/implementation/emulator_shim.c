#include "config.h"
#include <mono/metadata/appdomain.h>
#include <mono/metadata/class.h>
#include <mono/metadata/image.h>
#include <mono/metadata/object.h>
#include <mono/metadata/assembly.h>
#include <mono/metadata/debug-helpers.h>
#include <mono/jit/jit.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>


typedef struct {
    int isOffset;
    int number_;
    int* words;
    size_t words_len;
} Command;

typedef struct {
    MonoDomain* dom;
    MonoImage* im;
    MonoObject* emul;
} Emulator;

typedef enum {
    OK,
    NO_COMMANDS,
    INCORRECT_COMMAND,
    LOOP,
    END,
} ResultCode;

Emulator create_emulator() {
  Emulator instance;
  instance.dom = mono_jit_init("mtemu");
  MonoAssembly *mtemu_emu = mono_domain_assembly_open(
      instance.dom, PKGDATADIR "/engine.dll");
  instance.im = mono_assembly_get_image(mtemu_emu);
  /* TODO: currently broken for whatever reason, will dig deeper later */
  /* MonoClass *PortExtender = mono_class_from_name(im, "mtemu", "PortExtender"); */
  /* MonoObject *port_extender = mono_object_new(dom, PortExtender); */
  /* MonoMethodDesc* PortExtenderCtorDesc =
   * mono_method_desc_new("mtemu.PortExtender:.ctor()", 1); */
  /* MonoMethod* PortExtenderCtor =
   * mono_method_desc_search_in_class(PortExtenderCtorDesc, PortExtender); */
  /* mono_runtime_invoke(PortExtenderCtor, port_extender, NULL, NULL); */
  MonoClass *Emulator = mono_class_from_name(instance.im, "mtemu", "Emulator");
  instance.emul = mono_object_new(instance.dom, Emulator);
  // MonoMethodDesc* EmulatorCtorDesc =
  // mono_method_desc_new("mtemu.Emulator:.ctor(PortExtender)", 1);
  MonoMethodDesc *EmulatorCtorDesc =
      mono_method_desc_new("mtemu.Emulator:.ctor()", 1);
  MonoMethod *EmulatorCtor =
      mono_method_desc_search_in_class(EmulatorCtorDesc, Emulator);
  /* void* args[1]; */
  /* args[0] = &port_extender; */
  /* mono_runtime_invoke(EmulatorCtor, emulator, args, NULL); */
  mono_runtime_invoke(EmulatorCtor, instance.emul, NULL, NULL);
  return instance;
}

void emulator_reset(Emulator* inst) {
    MonoClass *emul_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ResetDesc =
        mono_method_desc_new("mtemu.Emulator:Reset()", 1);
    MonoMethod *Reset = mono_method_desc_search_in_class(ResetDesc, emul_class);
    mono_runtime_invoke(Reset, inst->emul, NULL, NULL);
}

MonoObject *emulator_get_command_managed(Emulator *inst, int32_t index) {
    MonoClass *emul_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetCommandDesc =
        mono_method_desc_new("mtemu.Emulator:GetCommand(int)", 1);
    MonoMethod *GetCommand =
        mono_method_desc_search_in_class(GetCommandDesc, emul_class);
    void *args[1];
    args[0] = &index;
    MonoObject *exception;
    MonoObject *res = mono_runtime_invoke(GetCommand, inst->emul, args, &exception);
    if (exception) {
        return exception;
    }
    return res;
}

Command command_unmanage(Emulator *inst, MonoObject *cmd_obj) {
  MonoClass *CommandClass = mono_object_get_class(cmd_obj);
  MonoClassField *isOffset =
      mono_class_get_field_from_name(CommandClass, "isOffset");
  MonoClassField *number_ =
      mono_class_get_field_from_name(CommandClass, "number_");
  MonoClassField *words_ =
      mono_class_get_field_from_name(CommandClass, "words_");
  Command cmd;
  mono_field_get_value(cmd_obj, isOffset, &cmd.isOffset);
  mono_field_get_value(cmd_obj, number_, &cmd.number_);
  MonoArray *words_arr =
      (MonoArray *)mono_field_get_value_object(inst->dom, words_, cmd_obj);
  cmd.words_len = mono_array_length(words_arr);
  cmd.words = malloc(cmd.words_len);
  for (size_t i = 0; i < cmd.words_len; ++i) {
    cmd.words[i] = mono_array_get(words_arr, int, i);
  }
  return cmd;
}

Command emulator_get_command(Emulator *inst, int32_t index) {
    return command_unmanage(inst, emulator_get_command_managed(inst, index));
}

MonoObject *command_manage(Emulator *inst, Command command) {
    MonoClass *CommandClass = mono_class_from_name_case(inst->im, "mtemu", "Command");
    MonoObject *CommandObject = mono_object_new(inst->dom, CommandClass);
    MonoClassField *isOffset =
      mono_class_get_field_from_name(CommandClass, "isOffset");
    MonoClassField *number_ =
      mono_class_get_field_from_name(CommandClass, "number_");
    MonoClassField *words_ =
      mono_class_get_field_from_name(CommandClass, "words_");
    mono_field_set_value(CommandObject, isOffset, &command.isOffset);
    mono_field_set_value(CommandObject, number_, &command.number_);
    MonoArray *words_array = mono_array_new(inst->dom, mono_get_int32_class(), command.words_len);
    for (size_t i = 0; i < command.words_len; ++i) {
        mono_array_set(words_array, int32_t, i, command.words[i]);
    }
    mono_field_set_value(CommandObject, words_, words_array);
    return CommandObject;
}

bool emulator_add_command(Emulator* inst, int index, Command command) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *AddCommandDesc = mono_method_desc_new("mtemu.Emulator:AddCommand(int, Command)", 1);
    MonoMethod *AddCommand = mono_method_desc_search_in_class(AddCommandDesc, emulator_class);
    void *args[2];
    args[0] = &index;
    args[1] = command_manage(inst, command);
    return mono_object_unbox(mono_runtime_invoke(AddCommand, inst->emul, args, NULL));
}

bool emulator_update_command(Emulator *inst, int index, Command command) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *UpdateCommandDesc = mono_method_desc_new("mtemu.Emulator:UpdateCommand(int, Command)", 1);
    MonoMethod *UpdateCommand = mono_method_desc_search_in_class(UpdateCommandDesc, emulator_class);
    void *args[2];
    args[0] = &index;
    args[1] = command_manage(inst, command);
    return mono_object_unbox(mono_runtime_invoke(UpdateCommand, inst->emul, args, NULL));
}

Command emulator_last_command(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *LastCommandDesc = mono_method_desc_new("mtemu.Emulator:LastCommand()", 1);
    MonoMethod *LastCommand = mono_method_desc_search_in_class(LastCommandDesc, emulator_class);
    return command_unmanage(inst, mono_runtime_invoke(LastCommand, inst->emul, NULL, NULL));
}

int* emulator_remove_command(Emulator *inst, int32_t index) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *RemoveCommandDesc = mono_method_desc_new("mtemu.Emulator:RemoveCommand(int)", 1);
    MonoMethod *RemoveCommand = mono_method_desc_search_in_class(RemoveCommandDesc, emulator_class);
    void *args[1];
    args[0] = &index;
    MonoObject *exception;
    mono_runtime_invoke(RemoveCommand, inst->emul, args, &exception);
    return (int*)exception;
}

int* emulator_commands_count(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *CommandsCountDesc = mono_method_desc_new("mtemu.Emulator:CommandsCount()", 1);
    MonoMethod *CommandCount = mono_method_desc_search_in_class(CommandsCountDesc, emulator_class);
    return mono_object_unbox(mono_runtime_invoke(CommandCount, inst->emul, NULL, NULL));
}

Command emulator_executed_command(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecutedCommandDesc = mono_method_desc_new("mtemu.Emulator:ExecutedCommand()", 1);
    MonoMethod *ExecutedCommand = mono_method_desc_search_in_class(ExecutedCommandDesc, emulator_class);
    return command_unmanage(inst, mono_runtime_invoke(ExecutedCommand, inst->emul, NULL, NULL));
}

ResultCode emulator_exec_one(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecOneDesc = mono_method_desc_new("mtemu.Emulator:ExecOne()", 1);
    MonoMethod *ExecOne = mono_method_desc_search_in_class(ExecOneDesc, emulator_class);
    return *(int*)mono_object_unbox(mono_runtime_invoke(ExecOne, inst->emul, NULL, NULL));
}

ResultCode emulator_exec_one_call(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecOneCallDesc = mono_method_desc_new("mtemu.Emulator:ExecOneCall()", 1);
    MonoMethod *ExecOneCall = mono_method_desc_search_in_class(ExecOneCallDesc, emulator_class);
    return *(int*)mono_object_unbox(mono_runtime_invoke(ExecOneCall, inst->emul, NULL, NULL));
}

ResultCode emulator_exec_all(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecAllDesc = mono_method_desc_new("mtemu.Emulator:ExecAll()", 1);
    MonoMethod *ExecAll = mono_method_desc_search_in_class(ExecAllDesc, emulator_class);
    return *(int*)mono_object_unbox(mono_runtime_invoke(ExecAll, inst->emul, NULL, NULL));
}
