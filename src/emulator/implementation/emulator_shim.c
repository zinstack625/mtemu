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
  mono_method_desc_free(EmulatorCtorDesc);
  mono_free_method(EmulatorCtor);
  return instance;
}

void emulator_reset(Emulator* inst) {
    MonoClass *emul_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ResetDesc =
        mono_method_desc_new("mtemu.Emulator:Reset()", 1);
    MonoMethod *Reset = mono_method_desc_search_in_class(ResetDesc, emul_class);
    mono_runtime_invoke(Reset, inst->emul, NULL, NULL);
    mono_method_desc_free(ResetDesc);
    mono_free_method(Reset);
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
    mono_method_desc_free(GetCommandDesc);
    mono_free_method(GetCommand);
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
  cmd.words = malloc(cmd.words_len * sizeof(int32_t));
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

bool emulator_add_command(Emulator* inst, int32_t index, Command command) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *AddCommandDesc = mono_method_desc_new("mtemu.Emulator:AddCommand(int, Command)", 1);
    MonoMethod *AddCommand = mono_method_desc_search_in_class(AddCommandDesc, emulator_class);
    void *args[2];
    args[0] = &index;
    args[1] = command_manage(inst, command);
    bool res = mono_object_unbox(mono_runtime_invoke(AddCommand, inst->emul, args, NULL));
    mono_method_desc_free(AddCommandDesc);
    mono_free_method(AddCommand);
    return res;
}

bool emulator_update_command(Emulator *inst, int32_t index, Command command) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *UpdateCommandDesc = mono_method_desc_new("mtemu.Emulator:UpdateCommand(int, Command)", 1);
    MonoMethod *UpdateCommand = mono_method_desc_search_in_class(UpdateCommandDesc, emulator_class);
    void *args[2];
    args[0] = &index;
    args[1] = command_manage(inst, command);
    bool res = mono_object_unbox(mono_runtime_invoke(UpdateCommand, inst->emul, args, NULL));
    mono_method_desc_free(UpdateCommandDesc);
    mono_free_method(UpdateCommand);
    return res;
}

Command emulator_last_command(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *LastCommandDesc = mono_method_desc_new("mtemu.Emulator:LastCommand()", 1);
    MonoMethod *LastCommand = mono_method_desc_search_in_class(LastCommandDesc, emulator_class);
    Command res = command_unmanage(inst, mono_runtime_invoke(LastCommand, inst->emul, NULL, NULL));
    mono_method_desc_free(LastCommandDesc);
    mono_free_method(LastCommand);
    return res;
}

int* emulator_remove_command(Emulator *inst, int32_t index) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *RemoveCommandDesc = mono_method_desc_new("mtemu.Emulator:RemoveCommand(int)", 1);
    MonoMethod *RemoveCommand = mono_method_desc_search_in_class(RemoveCommandDesc, emulator_class);
    void *args[1];
    args[0] = &index;
    MonoObject *exception;
    mono_runtime_invoke(RemoveCommand, inst->emul, args, &exception);
    mono_method_desc_free(RemoveCommandDesc);
    mono_free_method(RemoveCommand);
    return (int*)exception;
}

int32_t emulator_commands_count(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *CommandsCountDesc = mono_method_desc_new("mtemu.Emulator:CommandsCount()", 1);
    MonoMethod *CommandCount = mono_method_desc_search_in_class(CommandsCountDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(CommandCount, inst->emul, NULL, NULL));
    mono_method_desc_free(CommandsCountDesc);
    mono_free_method(CommandCount);
    return res;
}

Command emulator_executed_command(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecutedCommandDesc = mono_method_desc_new("mtemu.Emulator:ExecutedCommand()", 1);
    MonoMethod *ExecutedCommand = mono_method_desc_search_in_class(ExecutedCommandDesc, emulator_class);
    Command res = command_unmanage(inst, mono_runtime_invoke(ExecutedCommand, inst->emul, NULL, NULL));
    mono_method_desc_free(ExecutedCommandDesc);
    mono_free_method(ExecutedCommand);
    return res;
}

ResultCode emulator_exec_one(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecOneDesc = mono_method_desc_new("mtemu.Emulator:ExecOne()", 1);
    MonoMethod *ExecOne = mono_method_desc_search_in_class(ExecOneDesc, emulator_class);
    ResultCode res = *(int*)mono_object_unbox(mono_runtime_invoke(ExecOne, inst->emul, NULL, NULL));
    mono_method_desc_free(ExecOneDesc);
    mono_free_method(ExecOne);
    return res;
}

ResultCode emulator_exec_one_call(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecOneCallDesc = mono_method_desc_new("mtemu.Emulator:ExecOneCall()", 1);
    MonoMethod *ExecOneCall = mono_method_desc_search_in_class(ExecOneCallDesc, emulator_class);
    ResultCode res = *(int*)mono_object_unbox(mono_runtime_invoke(ExecOneCall, inst->emul, NULL, NULL));
    mono_method_desc_free(ExecOneCallDesc);
    mono_free_method(ExecOneCall);
    return res;
}

ResultCode emulator_exec_all(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExecAllDesc = mono_method_desc_new("mtemu.Emulator:ExecAll()", 1);
    MonoMethod *ExecAll = mono_method_desc_search_in_class(ExecAllDesc, emulator_class);
    ResultCode res = *(int*)mono_object_unbox(mono_runtime_invoke(ExecAll, inst->emul, NULL, NULL));
    mono_method_desc_free(ExecAllDesc);
    mono_free_method(ExecAll);
    return res;
}

int32_t emulator_get_next_index(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetNextIndexDesc = mono_method_desc_new("mtemu.Emulator:GetNextIndex()", 1);
    MonoMethod *GetNextIndex = mono_method_desc_search_in_class(GetNextIndexDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetNextIndex, inst->emul, NULL, NULL));
    mono_method_desc_free(GetNextIndexDesc);
    mono_free_method(GetNextIndex);
    return res;
}

int32_t emulator_get_prev_index(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPrevIndexDesc = mono_method_desc_new("mtemu.Emulator:GetPrevIndex()", 1);
    MonoMethod *GetPrevIndex = mono_method_desc_search_in_class(GetPrevIndexDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetPrevIndex, inst->emul, NULL, NULL));
    mono_method_desc_free(GetPrevIndexDesc);
    mono_free_method(GetPrevIndex);
    return res;
}

int32_t emulator_get_call_index(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetCallIndexDesc = mono_method_desc_new("mtemu.Emulator:GetCallIndex()", 1);
    MonoMethod *GetCallIndex = mono_method_desc_search_in_class(GetCallIndexDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetCallIndex, inst->emul, NULL, NULL));
    mono_method_desc_free(GetCallIndexDesc);
    mono_free_method(GetCallIndex);
    return res;
}

int32_t emulator_get_pc(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPCDesc = mono_method_desc_new("mtemu.Emulator:GetPC()", 1);
    MonoMethod *GetPC = mono_method_desc_search_in_class(GetPCDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetPC, inst->emul, NULL, NULL));
    mono_method_desc_free(GetPCDesc);
    mono_free_method(GetPC);
    return res;
}


int32_t emulator_set_pc(Emulator *inst, int32_t value) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *SetPCDesc = mono_method_desc_new("mtemu.Emulator:SetPC()", 1);
    MonoMethod *SetPC = mono_method_desc_search_in_class(SetPCDesc, emulator_class);
    void *args[1] = {&value};
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(SetPC, inst->emul, args, NULL));
    mono_method_desc_free(SetPCDesc);
    mono_free_method(SetPC);
    return res;
}

int32_t emulator_get_sp(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetSPDesc = mono_method_desc_new("mtemu.Emulator:GetSP()", 1);
    MonoMethod *GetSP = mono_method_desc_search_in_class(GetSPDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetSP, inst->emul, NULL, NULL));
    mono_method_desc_free(GetSPDesc);
    mono_free_method(GetSP);
    return res;
}

int32_t emulator_set_sp(Emulator *inst, int32_t value) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *SetSPDesc = mono_method_desc_new("mtemu.Emulator:SetSP()", 1);
    MonoMethod *SetSP = mono_method_desc_search_in_class(SetSPDesc, emulator_class);
    void *args[1] = {&value};
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(SetSP, inst->emul, args, NULL));
    mono_method_desc_free(SetSPDesc);
    mono_free_method(SetSP);
    return res;
}

int32_t emulator_get_stack_value(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetStackValueDesc = mono_method_desc_new("mtemu.Emulator:GetStackValue()", 1);
    MonoMethod *GetStackValue = mono_method_desc_search_in_class(GetStackValueDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetStackValue, inst->emul, NULL, NULL));
    mono_method_desc_free(GetStackValueDesc);
    mono_free_method(GetStackValue);
    return res;
}

int32_t emulator_get_mp(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetMPDesc = mono_method_desc_new("mtemu.Emulator:GetMP()", 1);
    MonoMethod *GetMP = mono_method_desc_search_in_class(GetMPDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetMP, inst->emul, NULL, NULL));
    mono_method_desc_free(GetMPDesc);
    mono_free_method(GetMP);
    return res;
}

int32_t emulator_get_port(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPortDesc = mono_method_desc_new("mtemu.Emulator:GetPort()", 1);
    MonoMethod *GetPort = mono_method_desc_search_in_class(GetPortDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetPort, inst->emul, NULL, NULL));
    mono_method_desc_free(GetPortDesc);
    mono_free_method(GetPort);
    return res;
}

int32_t emulator_get_mem_value(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetMemValueDesc = mono_method_desc_new("mtemu.Emulator:GetMemValue()", 1);
    MonoMethod *GetMemValue = mono_method_desc_search_in_class(GetMemValueDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetMemValue, inst->emul, NULL, NULL));
    mono_method_desc_free(GetMemValueDesc);
    mono_free_method(GetMemValue);
    return res;
}

int32_t emulator_get_reg_q(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetRegQDesc = mono_method_desc_new("mtemu.Emulator:GetRegQ()", 1);
    MonoMethod *GetRegQ = mono_method_desc_search_in_class(GetRegQDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetRegQ, inst->emul, NULL, NULL));
    mono_method_desc_free(GetRegQDesc);
    mono_free_method(GetRegQ);
    return res;
}

int32_t emulator_get_reg_value(Emulator *inst, int32_t index) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetRegValueDesc = mono_method_desc_new("mtemu.Emulator:GetRegValue()", 1);
    MonoMethod *GetRegValue = mono_method_desc_search_in_class(GetRegValueDesc, emulator_class);
    void *args[1] = {&index};
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetRegValue, inst->emul, args, NULL));
    mono_method_desc_free(GetRegValueDesc);
    mono_free_method(GetRegValue);
    return res;
}

int32_t emulator_get_f(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetFDesc = mono_method_desc_new("mtemu.Emulator:GetF()", 1);
    MonoMethod *GetF = mono_method_desc_search_in_class(GetFDesc, emulator_class);
    int32_t res = *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetF, inst->emul, NULL, NULL));
    mono_method_desc_free(GetFDesc);
    mono_free_method(GetF);
    return res;
}

int32_t emulator_get_y(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetYDesc = mono_method_desc_new("mtemu.Emulator:GetY()", 1);
    MonoMethod *GetY = mono_method_desc_search_in_class(GetYDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetY, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_reg_q(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPrevRegQDesc = mono_method_desc_new("mtemu.Emulator:GetPrevRegQ()", 1);
    MonoMethod *GetPrevRegQ = mono_method_desc_search_in_class(GetPrevRegQDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetPrevRegQ, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_reg_a(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPrevRegADesc = mono_method_desc_new("mtemu.Emulator:GetPrevRegA()", 1);
    MonoMethod *GetPrevRegA = mono_method_desc_search_in_class(GetPrevRegADesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetPrevRegA, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_reg_b(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPrevRegBDesc = mono_method_desc_new("mtemu.Emulator:GetPrevRegQ()", 1);
    MonoMethod *GetPrevRegB = mono_method_desc_search_in_class(GetPrevRegBDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetPrevRegB, inst->emul, NULL, NULL));
}

int32_t emulator_get_r(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetRDesc = mono_method_desc_new("mtemu.Emulator:GetR()", 1);
    MonoMethod *GetR = mono_method_desc_search_in_class(GetRDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetR, inst->emul, NULL, NULL));
}

int32_t emulator_get_s(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetSDesc = mono_method_desc_new("mtemu.Emulator:GetS()", 1);
    MonoMethod *GetS = mono_method_desc_search_in_class(GetSDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetS, inst->emul, NULL, NULL));
}

int32_t emulator_get_z(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetZDesc = mono_method_desc_new("mtemu.Emulator:GetZ()", 1);
    MonoMethod *GetZ = mono_method_desc_search_in_class(GetZDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetZ, inst->emul, NULL, NULL));
}

int32_t emulator_get_f3(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetF3Desc = mono_method_desc_new("mtemu.Emulator:GetF3()", 1);
    MonoMethod *GetF3 = mono_method_desc_search_in_class(GetF3Desc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetF3, inst->emul, NULL, NULL));
}

int32_t emulator_get_c4(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetC4Desc = mono_method_desc_new("mtemu.Emulator:GetC4()", 1);
    MonoMethod *GetC4 = mono_method_desc_search_in_class(GetC4Desc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetC4, inst->emul, NULL, NULL));
}

int32_t emulator_get_ovr(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetOVRDesc = mono_method_desc_new("mtemu.Emulator:GetOvr()", 1);
    MonoMethod *GetOVR = mono_method_desc_search_in_class(GetOVRDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetOVR, inst->emul, NULL, NULL));
}

int32_t emulator_get_g(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetGDesc = mono_method_desc_new("mtemu.Emulator:GetG()", 1);
    MonoMethod *GetG = mono_method_desc_search_in_class(GetGDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetG, inst->emul, NULL, NULL));
}

int32_t emulator_get_p(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetPDesc = mono_method_desc_new("mtemu.Emulator:GetP()", 1);
    MonoMethod *GetP = mono_method_desc_search_in_class(GetPDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(GetP, inst->emul, NULL, NULL));
}

MonoObject *call_manage(Emulator *inst, Call call) {
    MonoClass *CallClass = mono_class_from_name_case(inst->im, "mtemu", "Call");
    MonoObject *CallObject = mono_object_new(inst->dom, CallClass);
    MonoClassField *address_ =
      mono_class_get_field_from_name(CallClass, "isOffset");
    MonoClassField *comment_ =
      mono_class_get_field_from_name(CallClass, "number_");
    mono_field_set_value(CallObject, address_, &call.address_);
    mono_field_set_value(CallObject, comment_, mono_string_new_wrapper(call.comment_));
    return CallObject;
}

Call call_unmanage(Emulator *inst, MonoObject *cmd_obj) {
  MonoClass *CallClass = mono_object_get_class(cmd_obj);
  MonoClassField *address_ =
      mono_class_get_field_from_name(CallClass, "address_");
  MonoClassField *comment_ =
      mono_class_get_field_from_name(CallClass, "comment_");
  Call call;
  mono_field_get_value(cmd_obj, address_, &call.address_);
  call.comment_ =
      mono_string_to_utf8((MonoString*)mono_field_get_value_object(inst->dom, comment_, cmd_obj));
  return call;
}

void emulator_add_call(Emulator *inst, int32_t index, Call call) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *AddCallDesc = mono_method_desc_new("mtemu.Emulator:AddCall()", 1);
    MonoMethod *AddCall = mono_method_desc_search_in_class(AddCallDesc, emulator_class);
    void* args[2] = {&index, call_manage(inst, call)};
    mono_object_unbox(mono_runtime_invoke(AddCall, inst->emul, args, NULL));
}

Call emulator_get_call(Emulator *inst, int32_t index) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *GetCallDesc = mono_method_desc_new("mtemu.Emulator:GetCall()", 1);
    MonoMethod *GetCall = mono_method_desc_search_in_class(GetCallDesc, emulator_class);
    void* args[1] = {&index};
    return call_unmanage(inst, mono_runtime_invoke(GetCall, inst->emul, args, NULL));
}

void emulator_update_call(Emulator *inst, int32_t index, Call call) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *UpdateCallDesc = mono_method_desc_new("mtemu.Emulator:UpdateCall()", 1);
    MonoMethod *UpdateCall = mono_method_desc_search_in_class(UpdateCallDesc, emulator_class);
    void* args[2] = {&index, call_manage(inst, call)};
    mono_object_unbox(mono_runtime_invoke(UpdateCall, inst->emul, args, NULL));
}

void emulator_remove_call(Emulator *inst, int32_t index) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *RemoveCallDesc = mono_method_desc_new("mtemu.Emulator:RemoveCall()", 1);
    MonoMethod *RemoveCall = mono_method_desc_search_in_class(RemoveCallDesc, emulator_class);
    void* args[1] = {&index};
    mono_object_unbox(mono_runtime_invoke(RemoveCall, inst->emul, args, NULL));
}

int32_t emulator_calls_count(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *CallsCountDesc = mono_method_desc_new("mtemu.Emulator:CallsCount()", 1);
    MonoMethod *CallsCount = mono_method_desc_search_in_class(CallsCountDesc, emulator_class);
    return *(int32_t*)mono_object_unbox(mono_runtime_invoke(CallsCount, inst->emul, NULL, NULL));
}

Call emulator_last_call(Emulator *inst) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *LastCallDesc = mono_method_desc_new("mtemu.Emulator:LastCall()", 1);
    MonoMethod *LastCall = mono_method_desc_search_in_class(LastCallDesc, emulator_class);
    return call_unmanage(inst, mono_runtime_invoke(LastCall, inst->emul, NULL, NULL));
}

bool emulator_open_raw(Emulator *inst, uint8_t* bytes, size_t bytes_cnt) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *OpenRawDesc = mono_method_desc_new("mtemu.Emulator:OpenRaw(byte[])", 1);
    MonoMethod *OpenRaw = mono_method_desc_search_in_class(OpenRawDesc, emulator_class);
    MonoArray *managed_bytes = mono_array_new(inst->dom, mono_get_byte_class(), bytes_cnt);
    for (size_t i = 0; i < bytes_cnt; ++i) {
        mono_array_set(managed_bytes, uint8_t, i, bytes[i]);
    }
    void *args[1] = {managed_bytes};
    bool res = *(bool*)mono_object_unbox(mono_runtime_invoke(OpenRaw, inst->emul, args, NULL));
    mono_method_desc_free(OpenRawDesc);
    mono_free_method(OpenRaw);
    return res;
}

void emulator_export_raw(Emulator *inst, uint8_t** bytes, size_t* bytes_cnt) {
    MonoClass *emulator_class = mono_object_get_class(inst->emul);
    MonoMethodDesc *ExportRawDesc = mono_method_desc_new("mtemu.Emulator:ExportRaw()", 1);
    MonoMethod *ExportRaw = mono_method_desc_search_in_class(ExportRawDesc, emulator_class);
    MonoArray *managed_bytes = (MonoArray*)mono_runtime_invoke(ExportRaw, inst->emul, NULL, NULL);
    *bytes_cnt = mono_array_length(managed_bytes);
    *bytes = malloc(*bytes_cnt * sizeof(uint8_t));
    for (size_t i = 0; i < *bytes_cnt; ++i) {
        *(bytes[i]) = mono_array_get(managed_bytes, uint8_t, i);
    }
}

char* command_get_name(Emulator* inst, Command cmd) {
    MonoObject *command_managed = command_manage(inst, cmd);
    MonoClass *command_class = mono_object_get_class(command_managed);
    MonoMethodDesc *GetNameDesc = mono_method_desc_new("mtemu.Command:GetName()", 1);
    MonoMethod *GetName = mono_method_desc_search_in_class(GetNameDesc, command_class);
    return mono_string_to_utf8((MonoString*)mono_runtime_invoke(GetName, command_managed, NULL, NULL));
}

void free_obj(void* obj) {
    mono_free(obj);
}
