/* emulator_shim.c
 *
 * Copyright 2023 Anton Klimanov
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * 	http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */

#include "emulator_shim.h"
#include "config.h"
#include "mono/metadata/debug-helpers.h"
#include "mono/metadata/loader.h"
#include "mono/metadata/object-forward.h"
#include "mono/utils/mono-publib.h"
#include <stddef.h>
#include <mono/jit/jit.h>
#include <mono/metadata/appdomain.h>
#include <mono/metadata/assembly.h>
#include <mono/metadata/class.h>
#include <mono/metadata/image.h>
#include <mono/metadata/object.h>
#include <mono/metadata/mono-config.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

void init_emul_methods(Emulator* in, MonoClass* emulator_class) {
  MonoMethodDesc* desc = NULL;
  desc = mono_method_desc_new("mtemu.Emulator:Reset()", 1);
  in->methods.Reset = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetCommand(int)", 1);
  in->methods.GetCommand = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:AddUserCommand", 1);
  in->methods.AddCommand = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:UpdateCommand", 1);
  in->methods.UpdateCommand = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:LastCommand()", 1);
  in->methods.LastCommand = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:RemoveCommand(int)", 1);
  in->methods.RemoveCommand = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:CommandsCount()", 1);
  in->methods.CommandCount = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:ExecutedCommand()", 1);
  in->methods.ExecutedCommand = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:ExecOne()", 1);
  in->methods.ExecOne = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:ExecOneCall()", 1);
  in->methods.ExecOneCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:ExecAll()", 1);
  in->methods.ExecAll = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetNextIndex()", 1);
  in->methods.GetNextIndex = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetPrevIndex()", 1);
  in->methods.GetPrevIndex = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetCallIndex()", 1);
  in->methods.GetCallIndex = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetPC()", 1);
  in->methods.GetPC = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:SetPC()", 1);
  in->methods.SetPC = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetSP()", 1);
  in->methods.GetSP = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:SetSP()", 1);
  in->methods.SetSP = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetStackValue(int)", 1);
  in->methods.GetStackValue = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetStackLen()", 1);
  in->methods.GetStackLen = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMP()", 1);
  in->methods.GetMP = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetPort()", 1);
  in->methods.GetPort = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMemValue(int)", 1);
  in->methods.GetMemValue = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMemLength()", 1);
  in->methods.GetMemLength = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMem()", 1);
  in->methods.GetMem = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetRegQ()", 1);
  in->methods.GetRegQ = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetRegValue(int)", 1);
  in->methods.GetRegValue = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetF()", 1);
  in->methods.GetF = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetY()", 1);
  in->methods.GetY = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetPrevRegQ()", 1);
  in->methods.GetPrevRegQ = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetPrevRegA()", 1);
  in->methods.GetPrevRegA = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetPrevRegQ()", 1);
  in->methods.GetPrevRegB = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetR()", 1);
  in->methods.GetR = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetS()", 1);
  in->methods.GetS = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetZ()", 1);
  in->methods.GetZ = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetF3()", 1);
  in->methods.GetF3 = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetC4()", 1);
  in->methods.GetC4 = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetOvr()", 1);
  in->methods.GetOVR = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetG", 1);
  in->methods.GetG = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetP", 1);
  in->methods.GetP = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:AddCall", 1);
  in->methods.AddCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetCall", 1);
  in->methods.GetCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:UpdateCall", 1);
  in->methods.UpdateCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:RemoveCall", 1);
  in->methods.RemoveCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:CallsCount", 1);
  in->methods.CallsCount = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:LastCall", 1);
  in->methods.LastCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:OpenRaw(byte[])", 1);
  in->methods.OpenRaw = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:ExportRaw", 1);
  in->methods.ExportRaw = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:AddMapCall", 1);
  in->methods.AddMapCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:RemoveMapCall", 1);
  in->methods.RemoveMapCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:UpdateMapCall", 1);
  in->methods.UpdateMapCall = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMapCallName", 1);
  in->methods.GetMapCallName = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMapCallAddr", 1);
  in->methods.GetMapCallAddr = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:GetMapCallsCodes", 1);
  in->methods.GetMapCallCodes = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:InitLibrary", 1);
  in->methods.InitLibrary = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Emulator:Clone", 1);
  in->methods.Clone = mono_method_desc_search_in_class(desc, emulator_class);
  mono_method_desc_free(desc);
}

void init_command_methods(Emulator* in, MonoClass* command_class) {
  MonoMethodDesc* desc = NULL;

  desc = mono_method_desc_new("mtemu.Command:GetName()", 1);
  in->methods.GetName = mono_method_desc_search_in_class(desc, command_class);
  mono_method_desc_free(desc);

  desc = mono_method_desc_new("mtemu.Command:GetJumpName()", 1);
  in->methods.GetJumpName = mono_method_desc_search_in_class(desc, command_class);
  mono_method_desc_free(desc);
}

Emulator* create_emulator() {
  Emulator* instance = malloc(sizeof(Emulator));
  mono_config_parse (NULL);
  instance->dom = mono_jit_init("mtemu");
  MonoAssembly *mtemu_emu = mono_domain_assembly_open(
      instance->dom, PKGDATADIR "/engine.dll");
  instance->im = mono_assembly_get_image(mtemu_emu);
  MonoClass *PortExtender = mono_class_from_name(instance->im, "mtemu",
  "PortExtender");
  MonoObject *port_extender = mono_object_new(instance->dom, PortExtender);
  MonoMethodDesc* PortExtenderCtorDesc =
    mono_method_desc_new("mtemu.PortExtender:.ctor()", 1);
  instance->methods.PortExtenderCtor =
  mono_method_desc_search_in_class(PortExtenderCtorDesc, PortExtender);
  mono_runtime_invoke(instance->methods.PortExtenderCtor, port_extender, NULL, NULL);
  MonoClass *Emulator = mono_class_from_name(instance->im, "mtemu", "Emulator");
  instance->emul = mono_object_new(instance->dom, Emulator);
  MonoMethodDesc* EmulatorCtorDesc =
    mono_method_desc_new("mtemu.Emulator:.ctor(mtemu.PortExtender)", 1);
  instance->methods.EmulatorCtor =
      mono_method_desc_search_in_class(EmulatorCtorDesc, Emulator);
  void* args[1] = { &port_extender };
  mono_runtime_invoke(instance->methods.EmulatorCtor, instance->emul, args, NULL);
  mono_method_desc_free(EmulatorCtorDesc);
  init_emul_methods(instance, Emulator);

  MonoClass *command = mono_class_from_name(instance->im, "mtemu", "Command");
  init_command_methods(instance, command);
  instance->is_clone = false;
  return instance;
}

Emulator* clone_emulator(const Emulator *inst) {
  Emulator* clone = malloc(sizeof(Emulator));
  clone->dom = inst->dom;
  clone->methods = inst->methods;
  clone->im = inst->im;
  clone->emul = mono_runtime_invoke(inst->methods.Clone, inst->emul, NULL, NULL);
  clone->is_clone = true;
  return clone;
}

void emulator_swap(Emulator *lhs, Emulator *rhs) {
  MonoObject *temp = lhs->emul;
  lhs->emul = rhs->emul;
  rhs->emul = temp;
}

void destroy_emulator(Emulator* inst) {
  if (inst->is_clone) {
    free(inst);
    return;
  }
  mono_free_method(inst->methods.EmulatorCtor);
  mono_free_method(inst->methods.PortExtenderCtor);
  mono_free_method(inst->methods.Reset);
  mono_free_method(inst->methods.GetCommand);
  mono_free_method(inst->methods.AddCommand);
  mono_free_method(inst->methods.UpdateCommand);
  mono_free_method(inst->methods.LastCommand);
  mono_free_method(inst->methods.RemoveCommand);
  mono_free_method(inst->methods.CommandCount);
  mono_free_method(inst->methods.ExecutedCommand);
  mono_free_method(inst->methods.ExecOne);
  mono_free_method(inst->methods.ExecOneCall);
  mono_free_method(inst->methods.ExecAll);
  mono_free_method(inst->methods.GetNextIndex);
  mono_free_method(inst->methods.GetPrevIndex);
  mono_free_method(inst->methods.GetCallIndex);
  mono_free_method(inst->methods.GetPC);
  mono_free_method(inst->methods.SetPC);
  mono_free_method(inst->methods.GetSP);
  mono_free_method(inst->methods.SetSP);
  mono_free_method(inst->methods.GetStackValue);
  mono_free_method(inst->methods.GetStackLen);
  mono_free_method(inst->methods.GetMP);
  mono_free_method(inst->methods.GetPort);
  mono_free_method(inst->methods.GetMemValue);
  mono_free_method(inst->methods.GetMemLength);
  mono_free_method(inst->methods.GetMem);
  mono_free_method(inst->methods.GetRegQ);
  mono_free_method(inst->methods.GetRegValue);
  mono_free_method(inst->methods.GetF);
  mono_free_method(inst->methods.GetY);
  mono_free_method(inst->methods.GetPrevRegQ);
  mono_free_method(inst->methods.GetPrevRegA);
  mono_free_method(inst->methods.GetPrevRegB);
  mono_free_method(inst->methods.GetR);
  mono_free_method(inst->methods.GetS);
  mono_free_method(inst->methods.GetZ);
  mono_free_method(inst->methods.GetF3);
  mono_free_method(inst->methods.GetC4);
  mono_free_method(inst->methods.GetOVR);
  mono_free_method(inst->methods.GetG);
  mono_free_method(inst->methods.GetP);
  mono_free_method(inst->methods.AddCall);
  mono_free_method(inst->methods.GetCall);
  mono_free_method(inst->methods.UpdateCall);
  mono_free_method(inst->methods.RemoveCall);
  mono_free_method(inst->methods.CallsCount);
  mono_free_method(inst->methods.LastCall);
  mono_free_method(inst->methods.OpenRaw);
  mono_free_method(inst->methods.ExportRaw);
  mono_free_method(inst->methods.GetName);
  mono_free_method(inst->methods.GetJumpName);
  mono_free_method(inst->methods.AddMapCall);
  mono_free_method(inst->methods.RemoveMapCall);
  mono_free_method(inst->methods.UpdateMapCall);
  mono_free_method(inst->methods.GetMapCallName);
  mono_free_method(inst->methods.GetMapCallAddr);
  mono_free_method(inst->methods.GetMapCallCodes);
  mono_free_method(inst->methods.InitLibrary);
  mono_free_method(inst->methods.Clone);
  mono_jit_cleanup(inst->dom);
  free(inst);
}

void emulator_reset(Emulator *inst) {
  mono_runtime_invoke(inst->methods.Reset, inst->emul, NULL, NULL);
}

MonoObject *emulator_get_command_managed(Emulator *inst, int32_t index) {
  void *args[1];
  args[0] = &index;
  MonoObject *exception;
  MonoObject *res =
      mono_runtime_invoke(inst->methods.GetCommand, inst->emul, args, &exception);
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
  MonoClass *CommandClass =
      mono_class_from_name_case(inst->im, "mtemu", "Command");
  MonoObject *CommandObject = mono_object_new(inst->dom, CommandClass);
  MonoClassField *isOffset =
      mono_class_get_field_from_name(CommandClass, "isOffset");
  MonoClassField *number_ =
      mono_class_get_field_from_name(CommandClass, "number_");
  MonoClassField *words_ =
      mono_class_get_field_from_name(CommandClass, "words_");
  mono_field_set_value(CommandObject, isOffset, &command.isOffset);
  mono_field_set_value(CommandObject, number_, &command.number_);
  MonoArray *words_array =
      mono_array_new(inst->dom, mono_get_int32_class(), command.words_len);
  for (size_t i = 0; i < command.words_len; ++i) {
    mono_array_set(words_array, int32_t, i, command.words[i]);
  }
  mono_field_set_value(CommandObject, words_, words_array);
  return CommandObject;
}

bool emulator_add_command(Emulator *inst, int32_t index, Command command) {
  void *args[2];
  args[0] = &index;
  args[1] = command_manage(inst, command);
  bool res = mono_object_unbox(
      mono_runtime_invoke(inst->methods.AddCommand, inst->emul, args, NULL));
  return res;
}

bool emulator_update_command(Emulator *inst, int32_t index, Command command) {
  void *args[2];
  args[0] = &index;
  args[1] = command_manage(inst, command);
  bool res = mono_object_unbox(
      mono_runtime_invoke(inst->methods.UpdateCommand, inst->emul, args, NULL));
  return res;
}

Command emulator_last_command(Emulator *inst) {
  return command_unmanage(
      inst, mono_runtime_invoke(inst->methods.LastCommand, inst->emul, NULL, NULL));
}

int *emulator_remove_command(Emulator *inst, int32_t index) {
  void *args[1];
  args[0] = &index;
  MonoObject *exception;
  mono_runtime_invoke(inst->methods.RemoveCommand, inst->emul, args, &exception);
  return (int *)exception;
}

int32_t emulator_commands_count(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
    mono_runtime_invoke(inst->methods.CommandCount, inst->emul, NULL, NULL));
}

Command emulator_executed_command(Emulator *inst) {
  return command_unmanage(
      inst, mono_runtime_invoke(inst->methods.ExecutedCommand, inst->emul, NULL, NULL));
}

ResultCode emulator_exec_one(Emulator *inst) {
  return *(int *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.ExecOne, inst->emul, NULL, NULL));
}

ResultCode emulator_exec_one_call(Emulator *inst) {
  return (ResultCode)*(int *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.ExecOneCall, inst->emul, NULL, NULL));
}

ResultCode emulator_exec_all(Emulator *inst) {
  return (ResultCode)*(int *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.ExecAll, inst->emul, NULL, NULL));
}

int32_t emulator_get_next_index(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetNextIndex, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_index(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetPrevIndex, inst->emul, NULL, NULL));
}

int32_t emulator_get_call_index(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetCallIndex, inst->emul, NULL, NULL));
}

int32_t emulator_get_pc(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetPC, inst->emul, NULL, NULL));
}

int32_t emulator_set_pc(Emulator *inst, int32_t value) {
  void *args[1] = {&value};
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.SetPC, inst->emul, args, NULL));
}

int32_t emulator_get_sp(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetSP, inst->emul, NULL, NULL));
}

int32_t emulator_set_sp(Emulator *inst, int32_t value) {
  void *args[1] = {&value};
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.SetSP, inst->emul, args, NULL));
}

int32_t emulator_get_stack_value(Emulator *inst, int32_t addr) {
  void* args[1] = {&addr};
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetStackValue, inst->emul, args, NULL));
}

int32_t emulator_get_stack_length(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetStackLen, inst->emul, NULL, NULL));
}

int32_t emulator_get_mp(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetMP, inst->emul, NULL, NULL));
}

int32_t emulator_get_port(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetPort, inst->emul, NULL, NULL));
}

int32_t emulator_get_mem_value(Emulator *inst, int32_t ind) {
  void* args = { &ind };
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetMemValue, inst->emul, args, NULL));
}

int32_t emulator_get_mem_length(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetMemLength, inst->emul, NULL, NULL));
}

void emulator_get_mem(Emulator *inst, int32_t **memory, size_t *mem_cnt) {
  MonoArray *managed_memory =
      (MonoArray *)mono_runtime_invoke(inst->methods.GetMem, inst->emul, NULL, NULL);
  *mem_cnt = mono_array_length(managed_memory);

  int32_t *array_local = malloc(*mem_cnt * sizeof(int32_t));
  for (size_t i = 0; i < *mem_cnt; ++i) {
    array_local[i] = mono_array_get(managed_memory, int32_t, i);
  }
  *memory = array_local;
}

int32_t emulator_get_reg_q(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetRegQ, inst->emul, NULL, NULL));
}

int32_t emulator_get_reg_value(Emulator *inst, int32_t index) {
  void *args[1] = {&index};
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetRegValue, inst->emul, args, NULL));
}

int32_t emulator_get_f(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetF, inst->emul, NULL, NULL));
}

int32_t emulator_get_y(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetY, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_reg_q(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetPrevRegQ, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_reg_a(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetPrevRegA, inst->emul, NULL, NULL));
}

int32_t emulator_get_prev_reg_b(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetPrevRegB, inst->emul, NULL, NULL));
}

int32_t emulator_get_r(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetR, inst->emul, NULL, NULL));
}

int32_t emulator_get_s(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetS, inst->emul, NULL, NULL));
}

int32_t emulator_get_z(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetZ, inst->emul, NULL, NULL));
}

int32_t emulator_get_f3(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetF3, inst->emul, NULL, NULL));
}

int32_t emulator_get_c4(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetC4, inst->emul, NULL, NULL));
}

int32_t emulator_get_ovr(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetOVR, inst->emul, NULL, NULL));
}

int32_t emulator_get_g(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetG, inst->emul, NULL, NULL));
}

int32_t emulator_get_p(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.GetP, inst->emul, NULL, NULL));
}

MonoObject *call_manage(Emulator *inst, Call call) {
  MonoClass *CallClass = mono_class_from_name_case(inst->im, "mtemu", "Call");
  MonoObject *CallObject = mono_object_new(inst->dom, CallClass);
  MonoClassField *code_ =
      mono_class_get_field_from_name(CallClass, "code_");
  MonoClassField *arg0_ =
      mono_class_get_field_from_name(CallClass, "arg0_");
  MonoClassField* arg1_ =
      mono_class_get_field_from_name(CallClass, "arg1_");
  mono_field_set_value(CallObject, code_, &call.code_);
  mono_field_set_value(CallObject, arg0_, &call.arg0_);
  mono_field_set_value(CallObject, arg1_, &call.arg1_);
  return CallObject;
}

Call call_unmanage(Emulator * inst, MonoObject *cmd_obj) {
  MonoClass *CallClass = mono_object_get_class(cmd_obj);
  MonoClassField *code_ =
      mono_class_get_field_from_name(CallClass, "code_");
  MonoClassField *arg0_ =
      mono_class_get_field_from_name(CallClass, "arg0_");
  MonoClassField *arg1_ =
      mono_class_get_field_from_name(CallClass, "arg1_");
  Call call;
  mono_field_get_value(cmd_obj, code_, &call.code_);
  mono_field_get_value(cmd_obj, arg0_, &call.arg0_);
  mono_field_get_value(cmd_obj, arg1_, &call.arg1_);
  return call;
}

void emulator_add_call(Emulator *inst, int32_t index, Call call) {
  void *args[4] = {&index, &call.code_, &call.arg0_, &call.arg1_};
  mono_object_unbox(mono_runtime_invoke(inst->methods.AddCall, inst->emul, args, NULL));
}

Call emulator_get_call(Emulator *inst, int32_t index) {
  void *args[1] = {&index};
  return call_unmanage(inst, mono_runtime_invoke(inst->methods.GetCall, inst->emul, args, NULL));
}

void emulator_update_call(Emulator *inst, int32_t index, Call call) {
  void *args[4] = {&index, &call.code_, &call.arg0_, &call.arg1_};
  mono_runtime_invoke(inst->methods.UpdateCall, inst->emul, args, NULL);
}

void emulator_remove_call(Emulator *inst, int32_t index) {
  void *args[1] = {&index};
  mono_runtime_invoke(inst->methods.RemoveCall, inst->emul, args, NULL);
}

int32_t emulator_calls_count(Emulator *inst) {
  return *(int32_t *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.CallsCount, inst->emul, NULL, NULL));
}

bool emulator_add_map_call(Emulator *inst, int32_t code, const char* name, int32_t addr) {
  MonoString* name_ = mono_string_new(inst->dom, name);
  void *args[3] = {&code, name_, &addr};
  return *(bool*)mono_object_unbox(
    mono_runtime_invoke(inst->methods.AddMapCall, inst->emul, args, NULL));
}

bool emulator_remove_map_call(Emulator *inst, int32_t code) {
  void *args[1] = {&code};
  return *(bool*)mono_object_unbox(
    mono_runtime_invoke(inst->methods.RemoveMapCall, inst->emul, args, NULL));
}

bool emulator_update_map_call(Emulator *inst, int32_t code, const char* name, int32_t addr) {
  MonoString* name_ = mono_string_new(inst->dom, name);
  void *args[3] = {&code, name_, &addr};
  return *(bool*)mono_object_unbox(
    mono_runtime_invoke(inst->methods.UpdateMapCall, inst->emul, args, NULL));
}

int32_t* emulator_get_map_calls_codes(Emulator* inst, uint64_t* cnt) {
  MonoArray* codes = (MonoArray*)mono_runtime_invoke(inst->methods.GetMapCallCodes, inst->emul, NULL, NULL);
  *cnt = mono_array_length(codes);
  int32_t* local_codes = malloc(*cnt * sizeof(int32_t));
  for (uint64_t i = 0; i < *cnt; ++i) {
    local_codes[i] = mono_array_get(codes, int32_t, i);
  }
  return local_codes;
}

char* emulator_get_map_call_name(Emulator* inst, int32_t code) {
  void* args[1] = {&code};
  MonoObject* exception;
  MonoString * result = (MonoString*)mono_runtime_invoke(inst->methods.GetMapCallName, inst->emul, args, &exception);
  return mono_string_to_utf8(result);
}

int32_t emulator_get_map_call_addr(Emulator* inst, int32_t code) {
  void* args[1] = {&code};
  return *(int32_t*)mono_object_unbox(mono_runtime_invoke(inst->methods.GetMapCallAddr, inst->emul, args, NULL));
}

Call emulator_last_call(Emulator *inst) {
  return call_unmanage(
      inst, mono_runtime_invoke(inst->methods.LastCall, inst->emul, NULL, NULL));
}

bool emulator_open_raw(Emulator *inst, uint8_t *bytes, size_t bytes_cnt) {
  MonoArray *managed_bytes =
      mono_array_new(inst->dom, mono_get_byte_class(), bytes_cnt);
  for (size_t i = 0; i < bytes_cnt; ++i) {
    mono_array_set(managed_bytes, uint8_t, i, bytes[i]);
  }
  void *args[1] = {managed_bytes};
  return *(bool *)mono_object_unbox(
      mono_runtime_invoke(inst->methods.OpenRaw, inst->emul, args, NULL));
}

void emulator_export_raw(Emulator *inst, uint8_t **bytes, size_t *bytes_cnt) {
  MonoArray *managed_bytes =
      (MonoArray *)mono_runtime_invoke(inst->methods.ExportRaw, inst->emul, NULL, NULL);
  *bytes_cnt = mono_array_length(managed_bytes);

  uint8_t* bytes_local = malloc(*bytes_cnt * sizeof(uint8_t));
  for (size_t i = 0; i < *bytes_cnt; ++i) {
    bytes_local[i] = mono_array_get(managed_bytes, uint8_t, i);
  }
  *bytes = bytes_local;
}

void emulator_init_library(Emulator *inst) {
  mono_runtime_invoke(inst->methods.InitLibrary, inst->emul, NULL, NULL);
}

char *command_get_name(Emulator *inst, Command cmd) {
  MonoObject* command_managed = command_manage(inst, cmd);
  return mono_string_to_utf8(
      (MonoString *)mono_runtime_invoke(inst->methods.GetName, command_managed, NULL, NULL));
}

char *command_get_jump_name(Emulator *inst, Command cmd) {
  MonoObject *command_managed = command_manage(inst, cmd);
  return mono_string_to_utf8((MonoString *)mono_runtime_invoke(
      inst->methods.GetJumpName, command_managed, NULL, NULL));
}

void free_obj(void *obj) { mono_free(obj); }
