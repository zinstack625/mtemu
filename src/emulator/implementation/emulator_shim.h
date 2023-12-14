/* emulator_shim.h
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

#ifndef EMULATOR_SHIM_H_
#define EMULATOR_SHIM_H_

#include "mono/metadata/object-forward.h"
#include <mono/metadata/appdomain.h>
#include <mono/metadata/debug-helpers.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct {
  int32_t isOffset;
  int32_t number_;
  int32_t *words;
  size_t words_len;
} Command;

typedef struct {
  int32_t address_;
  char *comment_;
} Call;

typedef struct {
  MonoDomain *dom;
  MonoImage *im;
  MonoObject *emul;
  struct {
    MonoMethod* EmulatorCtor;
    MonoMethod* PortExtenderCtor;
    MonoMethod* Reset;
    MonoMethod* GetCommand;
    MonoMethod* AddCommand;
    MonoMethod* UpdateCommand;
    MonoMethod* LastCommand;
    MonoMethod* RemoveCommand;
    MonoMethod* CommandCount;
    MonoMethod* ExecutedCommand;
    MonoMethod* ExecOne;
    MonoMethod* ExecOneCall;
    MonoMethod* ExecAll;
    MonoMethod* GetNextIndex;
    MonoMethod* GetPrevIndex;
    MonoMethod* GetCallIndex;
    MonoMethod* GetPC;
    MonoMethod* SetPC;
    MonoMethod* GetSP;
    MonoMethod* SetSP;
    MonoMethod* GetStackValue;
    MonoMethod* GetStackLen;
    MonoMethod* GetMP;
    MonoMethod* GetPort;
    MonoMethod* GetMemValue;
    MonoMethod* GetMemLength;
    MonoMethod* GetMem;
    MonoMethod* GetRegQ;
    MonoMethod* GetRegValue;
    MonoMethod* GetF;
    MonoMethod* GetY;
    MonoMethod* GetPrevRegQ;
    MonoMethod* GetPrevRegA;
    MonoMethod* GetPrevRegB;
    MonoMethod* GetR;
    MonoMethod* GetS;
    MonoMethod* GetZ;
    MonoMethod* GetF3;
    MonoMethod* GetC4;
    MonoMethod* GetOVR;
    MonoMethod* GetG;
    MonoMethod* GetP;
    MonoMethod* AddCall;
    MonoMethod* GetCall;
    MonoMethod* UpdateCall;
    MonoMethod* RemoveCall;
    MonoMethod* CallsCount;
    MonoMethod* LastCall;
    MonoMethod* OpenRaw;
    MonoMethod* ExportRaw;
    MonoMethod* GetName;
    MonoMethod* GetJumpName;
  } methods;
} Emulator;

typedef enum {
  OK,
  NO_COMMANDS,
  INCORRECT_COMMAND,
  LOOP,
  END,
} ResultCode;

Emulator *create_emulator();
void destroy_emulator(Emulator *);
void emulator_reset(Emulator *);
Command emulator_get_command(Emulator *, int32_t);
bool emulator_add_command(Emulator *, int32_t, Command);
bool emulator_update_command(Emulator *, int32_t, Command);
Command emulator_last_command(Emulator *);
int32_t *emulator_remove_command(Emulator *, int32_t);
int32_t emulator_commands_count(Emulator *);
Command emulator_executed_command(Emulator *);
ResultCode emulator_exec_one(Emulator *);
ResultCode emulator_exec_one_call(Emulator *);
ResultCode emulator_exec_all(Emulator *);
int32_t emulator_get_next_index(Emulator *);
int32_t emulator_get_prev_index(Emulator *);
int32_t emulator_get_call_index(Emulator *);
int32_t emulator_get_pc(Emulator *);
int32_t emulator_set_pc(Emulator *, int32_t);
int32_t emulator_get_sp(Emulator *);
int32_t emulator_set_sp(Emulator *, int32_t);
int32_t emulator_get_stack_value(Emulator *, int32_t);
int32_t emulator_get_stack_length(Emulator *);
int32_t emulator_get_mp(Emulator *);
int32_t emulator_get_port(Emulator *);
int32_t emulator_get_mem_value(Emulator *, int32_t);
int32_t emulator_get_mem_length(Emulator *);
void emulator_get_mem(Emulator *, int32_t **, size_t *);
int32_t emulator_get_reg_q(Emulator *);
int32_t emulator_get_reg_value(Emulator *, int32_t);
int32_t emulator_get_f(Emulator *);
int32_t emulator_get_y(Emulator *);
int32_t emulator_get_prev_reg_q(Emulator *);
int32_t emulator_get_prev_reg_a(Emulator *);
int32_t emulator_get_prev_reg_b(Emulator *);
int32_t emulator_get_r(Emulator *);
int32_t emulator_get_s(Emulator *);
int32_t emulator_get_z(Emulator *);
int32_t emulator_get_f3(Emulator *);
int32_t emulator_get_c4(Emulator *);
int32_t emulator_get_ovr(Emulator *);
int32_t emulator_get_g(Emulator *);
int32_t emulator_get_p(Emulator *);
void emulator_add_call(Emulator *, int32_t, Call);
Call emulator_get_call(Emulator *, int32_t);
void emulator_update_call(Emulator *, int32_t, Call);
void emulator_remove_call(Emulator *, int32_t);
int32_t emulator_calls_count(Emulator *);
Call emulator_last_call(Emulator *);
bool emulator_open_raw(Emulator *, uint8_t *, size_t);
char *command_get_name(Emulator *, Command);
void free_obj(void *);

#endif // EMULATOR_SHIM_H_
