#include "emulator_shim.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
  Emulator* emul = create_emulator();
  FILE* mtemu_prog = fopen("/home/anton/mtemu_prog.mte", "rb");
  fseek(mtemu_prog, 0, SEEK_END);
  size_t fsize = ftell(mtemu_prog);
  fseek(mtemu_prog, 0, SEEK_SET);
  uint8_t* file_bytes = malloc(fsize * sizeof(uint8_t));
  fread(file_bytes, sizeof(uint8_t), fsize, mtemu_prog);
  fclose(mtemu_prog);
  emulator_open_raw(emul, file_bytes, fsize);
  for (int32_t i = 0; i < emulator_commands_count(emul); ++i) {
      Command cmd = emulator_get_command(emul, i);
      char* name = command_get_name(emul, cmd);
      printf("%s\n", name);
      free_obj(name);
  }
}
