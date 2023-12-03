using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;

namespace mtemu
{
    partial class Emulator
    {
        private PortExtender portExtender_;

        private int prevPc_;
        private int pc_;        // Pointer command
        private int callIndex_;
        private bool end_;

        private List<Command> commands_ = new List<Command>();
        private List<Call> calls_ = new List<Call>();
        private Dictionary<int, Tuple<string, int>> mapCalls_ = new Dictionary<int, Tuple<string, int>>(); // code => {name, address}

        private int sp_;        // Stack pointer
        private int[] stack_ = new int[stackSize_];

        private int regQ_;
        private int[] regCommon_;

        private IncType inc_;
        private int mp_;
        private int[] memory_ = new int[memSize_];

        private int devPtr_;

        private int prevRegA_;
        private int prevRegB_;
        private int prevRegQ_;
        private int r_;
        private int s_;

        private int f_;
        private int y_;

        private bool z_;
        private bool f3_;
        private bool c4_;
        private bool ovr_;
        private bool g_;
        private bool p_;

        private bool prevZ_;
        private bool prevF3_;
        private bool prevC4_;
        private bool prevOvr_;
        private bool prevG_;
        private bool prevP_;

        public void Reset()
        {
            prevPc_ = -1;
            pc_ = -1;
            callIndex_ = -1;
            end_ = false;

            sp_ = 0;
            regQ_ = 0;
            regCommon_ = new int[regSize_];
            inc_ = IncType.NO;
            mp_ = 0;
            devPtr_ = -1;

            prevRegA_ = 0;
            prevRegB_ = 0;
            prevRegQ_ = 0;
            r_ = 0;
            s_ = 0;

            f_ = 0;
            y_ = 0;

            z_ = false;
            f3_ = false;
            c4_ = false;
            ovr_ = false;
            g_ = false;
            p_ = false;

            prevZ_ = false;
            prevF3_ = false;
            prevC4_ = false;
            prevOvr_ = false;
            prevG_ = false;
            prevP_ = false;
        }

        public Emulator(PortExtender portExtender)
        {
            InitCommands();
            InitCalls();
            portExtender_ = portExtender;
            Reset();
        }

        private void InitCalls()
        {
            mapCalls_ = mapJumps_;
        }

        private void InitCommands()
        {
            foreach (Command cmd in callCommands)
            {
                AddLibCommand(cmd);
            }
        }

        private void BackupFlags_()
        {
            prevZ_ = z_;
            prevF3_ = f3_;
            prevC4_ = c4_;
            prevOvr_ = ovr_;
            prevG_ = g_;
            prevP_ = p_;
        }

        private void RestoreFlags()
        {
            z_ = prevZ_;
            f3_ = prevF3_;
            c4_ = prevC4_;
            ovr_ = prevOvr_;
            g_ = prevG_;
            p_ = prevP_;
        }

        private int GetLastCommandBeforOffset(int index)
        {
            while (index < CommandsCount())
            {
                if (GetCommand(index).isOffset) break;
                ++index;
            }
            return index;
        }

        private int GetOffset_(int index)
        {
            // Подсчет OffSet для команды по индексу index
            // index = commands[index - 1].GetNumber() + 1 || 0
            return index > 0 ? GetCommand(index - 1).GetNumber() + 1 : 0;
        }

        private void UpdateOffsets_(int first = 0)
        {
            // Обновление все OffSet с индекса first
            for (int i = first; i < CommandsCount(); ++i)
            {
                Command command = GetCommand(i);
                if (command.isOffset)
                {
                    command.SetNumber(command.GetNextAddr() - 1);
                }
                else
                {
                    command.SetNumber(GetOffset_(i));
                }
            }
        }

        public Command GetCommand(int index)
        {
            return commands_[index];
        }

        public bool AddUserCommand(int index, Command command)
        {
            if (!command.Check()) return false;

            if (command.isOffset)
            {
                command.SetNumber(command.GetNextAddr() - 1);
            }
            else
            {
                command.SetNumber(GetOffset_(index));
            }

            if (command.GetNumber() >= userProgramSize) return false;
            if (GetLastCommandBeforOffset(index + 1) - index - 1 + command.GetNumber() >= userProgramSize) return false;

            commands_.Insert(index, command);
            UpdateOffsets_(index + 1);
            return true;
        }

        private bool AddLibCommand(Command command)
        {
            int index = CommandsCount();
            if (!command.Check()) return false;

            if (command.isOffset)
            {
                command.SetNumber(command.GetNextAddr() - 1);
            }
            else
            {
                command.SetNumber(GetOffset_(index));
            }

            commands_.Insert(index, command);
            UpdateOffsets_(index + 1);
            return true;
        }

        public bool UpdateCommand(int index, Command command)
        {
            if (!command.Check()) return false;

            if (command.isOffset)
            {
                command.SetNumber(command.GetNextAddr() - 1);
            }
            else
            {
                command.SetNumber(GetOffset_(index));
            }

            if (command.GetNumber() >= userProgramSize) return false;
            if (GetLastCommandBeforOffset(index + 1) - index - 1 + command.GetNumber() >= userProgramSize) return false;

            commands_[index] = command;
            UpdateOffsets_(index + 1);
            return true;
        }

        public bool RemoveCommand(int index)
        {
            Command command = GetCommand(index);
            if (command.GetNumber() >= userProgramSize || command.isOffset && command.GetNumber() >= userProgramSize - 1) return false;
            commands_.RemoveAt(index);
            UpdateOffsets_(index);
            return false;
        }

        public void MoveCommandUp(int index)
        {
            if (index <= 0)
            {
                return;
            }
            commands_.Insert(index - 1, commands_[index]);
            commands_.RemoveAt(index + 1);
            UpdateOffsets_(index - 1);
        }

        public void MoveCommandDown(int index)
        {
            if (index == commands_.Count - 1)
            {
                return;
            }
            commands_.Insert(index + 2, commands_[index]);
            commands_.RemoveAt(index);
            UpdateOffsets_(index);
        }

        public int CommandsCount()
        {
            return commands_.Count();
        }

        private int GetIndex_(int addr)
        {
            int curr = 0;
            foreach (Command command in commands_)
            {
                if (command.GetNumber() - addr == 0 && !command.isOffset)
                {
                    return curr;
                }
                ++curr;
            }
            return -1;
        }

        public Command ExecutedCommand()
        {
            int i = GetIndex_(prevPc_);
            if (i == -1)
            {
                return incorrectCommand_;
            }
            return commands_[i];
        }

        private Command Prev_()
        {
            int i = GetIndex_(prevPc_);
            if (i == -1)
            {
                return incorrectCommand_;
            }
            return commands_[i];
        }

        private Command Current_()
        {
            int i = GetIndex_(pc_);
            if (i == -1)
            {
                return incorrectCommand_;
            }
            return commands_[i];
        }

        private int GetStackAddr_(int sp)
        {
            return (sp + stackSize_) % stackSize_;
        }

        private int GetAddrByCode(int code)
        {
            if (mapCalls_.ContainsKey(code))
            {
                Tuple<string, int> nameAddr = mapCalls_[code];
                return nameAddr.Item2;
            }
            return -1;
        }

        public string GetNameByCode(int code)
        {
            if (mapCalls_.ContainsKey(code))
            {
                Tuple<string, int> nameAddr = mapCalls_[code];
                return nameAddr.Item1;
            }
            return "";
        }

        private void Jump_()
        {
            prevPc_ = pc_;

            switch (Current_().GetJumpType())
            {
                case JumpType.END:
                    if (calls_.Count <= 0 || calls_.Count <= callIndex_ )
                    {
                        end_ = true;
                        return;
                    }
                    Call call = calls_[callIndex_];
                    if (call.GetAltCommandAddress())
                    {
                        RestoreFlags();

                        switch (call.GetFlag())
                        {
                            case JumpType.JZ:
                                if (prevZ_)
                                {
                                    callIndex_ = call.GetArg0();
                                    break;
                                }
                                ++callIndex_;
                                break;
                            case JumpType.JC4:
                                if (prevC4_)
                                {
                                    callIndex_ = call.GetArg0();
                                    break;
                                }
                                ++callIndex_;
                                break;
                            case JumpType.JNZ:
                                if (!prevZ_)
                                {
                                    callIndex_ = call.GetArg0();
                                    break;
                                }
                                ++callIndex_;
                                break;
                            case JumpType.JSNC4:
                                if (!prevC4_)
                                {
                                    callIndex_ = call.GetArg0();
                                    break;
                                }
                                ++callIndex_;
                                break;
                            case JumpType.JMP:
                                callIndex_ = call.GetArg0();
                                break;
                        }
                    } 
                    else
                    {
                        ++callIndex_;
                    }

                    if (calls_.Count > 0 && callIndex_ <= calls_.Count)
                    {
                        pc_ = GetAddrByCode(call.GetCode());
                        return;
                    }
                    end_ = true;
                    return;
                case JumpType.JMP:
                    pc_ = Current_().GetNextAddr();
                    return;

                case JumpType.JNXT:
                    ++pc_;
                    return;

                case JumpType.JNZ:
                    if (!prevZ_)
                    {
                        pc_ = Current_().GetNextAddr();
                        break;
                    }
                    ++pc_;
                    break;

                case JumpType.JZ:
                    if (prevZ_)
                    {
                        pc_ = Current_().GetNextAddr();
                        break;
                    }
                    ++pc_;
                    break;

                case JumpType.JF3:
                    if (prevF3_)
                    {
                        pc_ = Current_().GetNextAddr();
                        break;
                    }
                    ++pc_;
                    break;

                case JumpType.JOVR:
                    if (prevOvr_)
                    {
                        pc_ = Current_().GetNextAddr();
                        break;
                    }
                    ++pc_;
                    break;

                case JumpType.JC4:
                    if (prevC4_)
                    {
                        pc_ = Current_().GetNextAddr();
                        break;
                    }
                    ++pc_;
                    break;

                case JumpType.CALL:
                    stack_[sp_] = pc_ + 1;
                    sp_ = GetStackAddr_(sp_ + 1);
                    pc_ = Current_().GetNextAddr();
                    return;

                case JumpType.RET:
                    sp_ = GetStackAddr_(sp_ - 1);
                    pc_ = stack_[sp_];
                    return;

                case JumpType.JSP:
                    pc_ = stack_[GetStackAddr_(sp_ - 1)];
                    return;

                case JumpType.PUSH:
                    stack_[sp_] = pc_ + 1;
                    sp_ = GetStackAddr_(sp_ + 1);
                    ++pc_;
                    return;

                case JumpType.POP:
                    sp_ = GetStackAddr_(sp_ - 1);
                    ++pc_;
                    return;

                case JumpType.CLNZ:
                    if (!prevZ_)
                    {
                        stack_[sp_] = pc_ + 1;
                        sp_ = GetStackAddr_(sp_ + 1);
                        pc_ = Current_().GetNextAddr();
                        break;
                    }
                    ++pc_;
                    break;

                case JumpType.JSNZ:
                    if (!prevZ_)
                    {
                        pc_ = stack_[GetStackAddr_(sp_ - 1)];
                        break;
                    }
                    sp_ = GetStackAddr_(sp_ - 1);
                    ++pc_;
                    break;

                case JumpType.JSNC4:
                    if (!prevC4_)
                    {
                        pc_ = stack_[GetStackAddr_(sp_ - 1)];
                        break;
                    }
                    sp_ = GetStackAddr_(sp_ - 1);
                    ++pc_;
                    break;
            }

            RestoreFlags();
        }

        private void CountFlags_(FuncType alu)
        {
            bool c0 = (int)alu >= 8;

            int r = r_;
            int s = s_;
            switch (alu)
            {
                case FuncType.S_MINUS_R_MINUS_1:
                case FuncType.S_MINUS_R:
                case FuncType.NO_R_AND_S:
                case FuncType.R_XOR_S:
                    r = Helpers.Mask(~r);
                    break;
                case FuncType.R_MINUS_S_MINUS_1:
                case FuncType.R_MINUS_S:
                    s = Helpers.Mask(~s);
                    break;
            }

            int p = r | s;
            int g = r & s;
            bool p30 = p == 15;
            bool g30 = g > 0;

            switch (alu)
            {
                case FuncType.R_PLUS_S:
                case FuncType.R_PLUS_S_PLUS_1:
                case FuncType.S_MINUS_R_MINUS_1:
                case FuncType.S_MINUS_R:
                case FuncType.R_MINUS_S_MINUS_1:
                case FuncType.R_MINUS_S:
                    p_ = !p30;

                    bool g1 = Helpers.IsBitSet(g, 1) || (Helpers.IsBitSet(p, 1) && Helpers.IsBitSet(g, 0));
                    bool g2 = Helpers.IsBitSet(g, 2) || (Helpers.IsBitSet(p, 2) && g1);
                    bool g3 = Helpers.IsBitSet(g, 3) || (Helpers.IsBitSet(p, 3) && g2);
                    g_ = !g3;

                    bool c1 = Helpers.IsBitSet(g, 0) || (Helpers.IsBitSet(p, 0) && c0);
                    bool c2 = Helpers.IsBitSet(g, 1) || (Helpers.IsBitSet(p, 1) && c1);
                    bool c3 = Helpers.IsBitSet(g, 2) || (Helpers.IsBitSet(p, 2) && c2);
                    bool c4 = Helpers.IsBitSet(g, 3) || (Helpers.IsBitSet(p, 3) && c3);
                    c4_ = c4;
                    ovr_ = c3 != c4;

                    break;
                case FuncType.R_OR_S:
                    p_ = false;
                    g_ = p30;
                    c4_ = !p30 || c0;
                    ovr_ = c4_;
                    break;
                case FuncType.R_AND_S:
                case FuncType.NO_R_AND_S:
                    p_ = false;
                    g_ = !g30;
                    c4_ = g30 || c0;
                    ovr_ = c4_;
                    break;
                case FuncType.R_XOR_S:
                case FuncType.R_EQ_S:
                    p_ = g30;

                    bool g_1 = Helpers.IsBitSet(g, 1) || (Helpers.IsBitSet(p, 1) && Helpers.IsBitSet(p, 0));
                    bool g_2 = Helpers.IsBitSet(g, 2) || (Helpers.IsBitSet(p, 2) && g_1);
                    bool g_3 = Helpers.IsBitSet(g, 3) || (Helpers.IsBitSet(p, 3) && g_2);
                    g_ = g_3;

                    bool c4_1 = Helpers.IsBitSet(g, 1) || (Helpers.IsBitSet(p, 1) && Helpers.IsBitSet(p, 0) && (Helpers.IsBitSet(g, 0) || !c0));
                    bool c4_2 = Helpers.IsBitSet(g, 2) || (Helpers.IsBitSet(p, 2) && c4_1);
                    bool c4_3 = Helpers.IsBitSet(g, 3) || (Helpers.IsBitSet(p, 3) && c4_2);
                    c4_ = !c4_3;

                    p = Helpers.Mask(~p);
                    g = Helpers.Mask(~g);
                    bool ovr_0 = Helpers.IsBitSet(p, 0) || (Helpers.IsBitSet(g, 0) && c0);
                    bool ovr_1 = Helpers.IsBitSet(p, 1) || (Helpers.IsBitSet(g, 1) && ovr_0);
                    bool ovr_2 = Helpers.IsBitSet(p, 2) || (Helpers.IsBitSet(g, 2) && ovr_1);
                    bool ovr_3 = Helpers.IsBitSet(p, 3) || (Helpers.IsBitSet(g, 3) && ovr_2);
                    ovr_ = ovr_2 != ovr_3;

                    break;
            }

            f3_ = Helpers.IsBitSet(f_, Command.WORD_SIZE - 1);
            z_ = f_ == 0;
        }

        private void ExecMtCommand_()
        {
            FromType from = Current_().GetFromType();
            FuncType alu = Current_().GetFuncType();
            ToType to = Current_().GetToType();
            ShiftType shift = Current_().GetShiftType();

            int a = Current_().GetRawValue(WordType.A);
            int b = Current_().GetRawValue(WordType.B);
            int d = Current_().GetRawValue(WordType.D);

            prevRegQ_ = regQ_;
            prevRegA_ = regCommon_[a];
            prevRegB_ = regCommon_[b];

            switch (from)
            {
                case FromType.A_AND_PQ:
                    r_ = regCommon_[a];
                    s_ = regQ_;
                    break;
                case FromType.A_AND_B:
                    r_ = regCommon_[a];
                    s_ = regCommon_[b];
                    break;
                case FromType.ZERO_AND_Q:
                    r_ = 0;
                    s_ = regQ_;
                    break;
                case FromType.ZERO_AND_B:
                    r_ = 0;
                    s_ = regCommon_[b];
                    break;
                case FromType.ZERO_AND_A:
                    r_ = 0;
                    s_ = regCommon_[a];
                    break;
                case FromType.D_AND_A:
                    r_ = d;
                    s_ = regCommon_[a];
                    break;
                case FromType.D_AND_Q:
                    r_ = d;
                    s_ = regQ_;
                    break;
                case FromType.D_AND_ZERO:
                    r_ = d;
                    s_ = 0;
                    break;
            }

            switch (alu)
            {
                case FuncType.R_PLUS_S:
                    f_ = r_ + s_;
                    break;
                case FuncType.R_PLUS_S_PLUS_1:
                    f_ = r_ + s_ + 1;
                    break;
                case FuncType.S_MINUS_R_MINUS_1:
                    f_ = s_ + Helpers.Mask(~r_);
                    break;
                case FuncType.S_MINUS_R:
                    f_ = s_ + Helpers.Mask(~r_) + 1;
                    break;
                case FuncType.R_MINUS_S_MINUS_1:
                    f_ = r_ + Helpers.Mask(~s_);
                    break;
                case FuncType.R_MINUS_S:
                    f_ = r_ + Helpers.Mask(~s_) + 1;
                    break;
                case FuncType.R_OR_S:
                    f_ = r_ | s_;
                    break;
                case FuncType.R_AND_S:
                    f_ = r_ & s_;
                    break;
                case FuncType.NO_R_AND_S:
                    f_ = Helpers.Mask(~r_) & s_;
                    break;
                case FuncType.R_XOR_S:
                    f_ = r_ ^ s_;
                    break;
                case FuncType.R_EQ_S:
                    f_ = Helpers.Mask(~(r_ ^ s_));
                    break;
            }
            f_ = Helpers.Mask(f_);

            CountFlags_(alu);

            int qLow = Helpers.GetBit(regQ_, 0);
            int qHigh = Helpers.GetBit(regQ_, Command.WORD_SIZE - 1);
            int fLow = Helpers.GetBit(f_, 0);
            int fHigh = Helpers.GetBit(f_, Command.WORD_SIZE - 1);

            switch (to)
            {
                case ToType.F_IN_Q:
                    regQ_ = f_;
                    break;
                case ToType.NO_LOAD:
                    break;
                case ToType.F_IN_B_AND_A_IN_Y:
                case ToType.F_IN_B:
                    regCommon_[b] = f_;
                    break;
                case ToType.SR_F_IN_B_AND_SR_Q_IN_Q:
                    regQ_ = regQ_ >> 1;
                    regCommon_[b] = f_ >> 1;
                    switch (shift)
                    {
                        case ShiftType.CYCLE:
                            regCommon_[b] |= fLow << 3;
                            regQ_ |= qLow << 3;
                            break;
                        case ShiftType.CYCLE_DOUBLE:
                            regCommon_[b] |= qLow << 3;
                            regQ_ |= fLow << 3;
                            break;
                        case ShiftType.ARITHMETIC_DOUBLE:
                            regCommon_[b] |= fHigh << 3;
                            regQ_ |= fLow << 3;
                            break;
                    }
                    break;
                case ToType.SR_F_IN_B:
                    regCommon_[b] = f_ >> 1;
                    switch (shift)
                    {
                        case ShiftType.CYCLE:
                            regCommon_[b] |= fLow << 3;
                            break;
                    }
                    break;
                case ToType.SL_F_IN_B_AND_SL_Q_IN_Q:
                    regQ_ = Helpers.Mask(regQ_ << 1);
                    regCommon_[b] = Helpers.Mask(f_ << 1);
                    switch (shift)
                    {
                        case ShiftType.CYCLE:
                            regCommon_[b] |= fHigh;
                            regQ_ |= qHigh;
                            break;
                        case ShiftType.CYCLE_DOUBLE:
                            regCommon_[b] |= qHigh;
                            regQ_ |= fHigh;
                            break;
                        case ShiftType.ARITHMETIC_DOUBLE:
                            regCommon_[b] |= qHigh;
                            break;
                    }
                    break;
                case ToType.SL_F_IN_B:
                    regCommon_[b] = Helpers.Mask(f_ << 1);
                    switch (shift)
                    {
                        case ShiftType.CYCLE:
                            regCommon_[b] |= fHigh;
                            break;
                    }
                    break;
            }

            if (to == ToType.F_IN_B_AND_A_IN_Y)
            {
                y_ = regCommon_[a];
            }
            else
            {
                y_ = f_;
            }
        }

        private void SetMemPtr_()
        {
            inc_ = Current_().GetIncType();
            mp_ = (Current_().GetRawValue(WordType.A) << 4) + Current_().GetRawValue(WordType.B);
        }

        // 4 бита адреса устройства, подключенного к порту
        private int GetDeviceAddress()
        {
            return Current_().GetRawValue(WordType.D);
        }


        private PortExtender.Port GetPort(DataPointerType type)
        {
            if (devPtr_ == -1)
            {
                return PortExtender.Port.PORT_UNKNOWN;
            }

            var val = devPtr_ << 2;

            switch (type)
            {
                case DataPointerType.LOW_4_BIT:
                    val |= 1;
                    break;
                case DataPointerType.HIGH_4_BIT:
                    val |= 2;
                    break;
                case DataPointerType.FULL_8_BIT:
                    val |= 3;
                    break;
            }

            var val_b = Convert.ToByte(val);

            if (Enum.IsDefined(typeof(PortExtender.Port), val_b))
                return (PortExtender.Port) val_b;

            return PortExtender.Port.PORT_UNKNOWN;
        }

        private void SetDevicePtr_()
        {
            devPtr_ = Current_().GetRawValue(WordType.A);
        }

        private void LoadData_()
        {
            FuncType func = Current_().GetFuncType();
            DataPointerType pointerType = Current_().GetPointerType();
            int a = Current_().GetRawValue(WordType.A);
            int b = Current_().GetRawValue(WordType.B);
            switch (func) {
            case FuncType.STORE_MEMORY:
                switch (pointerType) {
                case DataPointerType.LOW_4_BIT:
                    memory_[mp_] = Helpers.MakeByte(
                        Helpers.HighNibble(memory_[mp_]),
                        regCommon_[b]);
                    break;
                case DataPointerType.HIGH_4_BIT:
                    memory_[mp_] = Helpers.MakeByte(
                        regCommon_[a],
                        Helpers.LowNibble(memory_[mp_]));
                    break;
                case DataPointerType.FULL_8_BIT:
                    memory_[mp_] = Helpers.MakeByte(
                        regCommon_[a],
                        regCommon_[b]);
                    break;
                }
                break;
            case FuncType.LOAD_MEMORY:
                switch (pointerType) {
                case DataPointerType.LOW_4_BIT:
                    regCommon_[b] = Helpers.LowNibble(memory_[mp_]);
                    break;
                case DataPointerType.HIGH_4_BIT:
                    regCommon_[a] = Helpers.HighNibble(memory_[mp_]);
                    break;
                case DataPointerType.FULL_8_BIT:
                    regCommon_[a] = Helpers.HighNibble(memory_[mp_]);
                    regCommon_[b] = Helpers.LowNibble(memory_[mp_]);
                    break;
                }
                break;
            case FuncType.STORE_DEVICE:
                    {
                        PortExtender.Port Port = GetPort(pointerType);
                        int Addr = GetDeviceAddress();
                        if (Port != PortExtender.Port.PORT_UNKNOWN)
                        {
                            byte tmp_w = 0;

                            switch (pointerType)
                            {
                                case DataPointerType.LOW_4_BIT:
                                    tmp_w = Helpers.MakeLowNibble(regCommon_[b]);
                                    break;
                                case DataPointerType.HIGH_4_BIT:
                                    tmp_w = Helpers.MakeHighNibble(regCommon_[a]);
                                    break;
                                case DataPointerType.FULL_8_BIT:
                                    tmp_w = Helpers.MakeByte(regCommon_[a], regCommon_[b]);
                                    break;
                            }

                            portExtender_.WritePort(Addr, Port, pointerType, tmp_w);
                        }
                        else
                        {
                            System.Windows.Forms.MessageBox.Show(
                                "Вы - болван, у Вас проблема с кодом (генетическим), проверьтесь у врача!\nТакие упорствующие индивиды не должны размножаться!",
                                "Невозможно записать в это устройство",
                                System.Windows.Forms.MessageBoxButtons.OK,
                                System.Windows.Forms.MessageBoxIcon.Error
                            );
                        }
                        break;
                    }
            case FuncType.LOAD_DEVICE:
                    {
                        PortExtender.Port Port = GetPort(pointerType);
                        int Addr = GetDeviceAddress();
                        if (Port != PortExtender.Port.PORT_UNKNOWN)
                        {
                            byte tmp_r;
                            tmp_r = portExtender_.ReadPort(Addr, Port, pointerType);
                            switch (pointerType)
                            {
                                case DataPointerType.LOW_4_BIT:
                                    regCommon_[b] = Helpers.LowNibble(tmp_r);
                                    break;
                                case DataPointerType.HIGH_4_BIT:
                                    regCommon_[a] = Helpers.HighNibble(tmp_r);
                                    break;
                                case DataPointerType.FULL_8_BIT:
                                    regCommon_[a] = Helpers.HighNibble(tmp_r);
                                    regCommon_[b] = Helpers.LowNibble(tmp_r);
                                    break;
                            }
                        }
                        else
                        {
                            System.Windows.Forms.MessageBox.Show(
                                "Вы - болван, у Вас проблема с кодом (генетическим), проверьтесь у врача!\nТакие упорствующие индивиды не должны размножаться!",
                                "Невозможно прочитать из этого устройства",
                                System.Windows.Forms.MessageBoxButtons.OK,
                                System.Windows.Forms.MessageBoxIcon.Error
                            );
                        }
                        break;
                    }
            }
            if (func == FuncType.STORE_MEMORY || func == FuncType.LOAD_MEMORY)
            {
                switch (inc_)
                {
                    case IncType.PLUS:
                        ++mp_;
                        mp_ %= memSize_;
                        break;
                    case IncType.MINUS:
                        --mp_;
                        mp_ %= memSize_;
                        break;
                }
            }
        }

        public ResultCode ExecOne()
        {
            if (commands_.Count() == 0)
            {
                return ResultCode.NoCommands;
            }

            if (end_)
            {
                return ResultCode.End;
            }

            if (pc_ == -1)
            {
                if (calls_.Count > 0)
                {
                    Call call = calls_[0];
                    memory_[0] = call.GetArg0();
                    memory_[1] = call.GetArg1();
                    pc_ = GetAddrByCode(call.GetCode());
                    callIndex_ = 1;
                }
                else
                {
                    pc_ = 0;
                }
                return ResultCode.Ok;
            }

            if (!Current_().Check())
            {
                return ResultCode.IncorrectCommand;
            }

            // Save flags to restore then after command exec
            BackupFlags_();

            switch (Current_().GetCommandView())
            {
                case ViewType.MT_COMMAND:
                    ExecMtCommand_();
                    break;
                case ViewType.MEMORY_POINTER:
                    SetMemPtr_();
                    break;
                case ViewType.DEVICE_POINTER:
                    SetDevicePtr_();
                    break;
                case ViewType.LOAD_HIGH_4BIT:
                case ViewType.LOAD_LOW_4BIT:
                case ViewType.LOAD_8BIT:
                    LoadData_();
                    break;
            }

            Jump_();
            pc_ %= programSize_;

            return ResultCode.Ok;
        }

        public ResultCode ExecOneCall()
        {
            int oldIndex = callIndex_;
            for (int i = 0; i < maxAutoCount_; ++i)
            {
                ResultCode rc = ExecOne();
                if (rc != ResultCode.Ok)
                {
                    return rc;
                }
                if (Prev_().GetJumpType() == JumpType.END || callIndex_ != oldIndex || prevPc_ == pc_)
                {
                    return ResultCode.Ok;
                }
            }
            return ResultCode.Loop;
        }

        public ResultCode ExecAll()
        {
            for (int i = 0; i < maxAutoCount_; ++i)
            {
                ResultCode rc = ExecOne();
                if (rc != ResultCode.Ok)
                {
                    return rc;
                }
                if (callIndex_ >= calls_.Count || prevPc_ == pc_)
                {
                    return ResultCode.Ok;
                }
            }
            return ResultCode.Loop;
        }

        public int GetPrevIndex()
        {
            return GetIndex_(prevPc_);
        }

        public int GetNextIndex()
        {
            return GetIndex_(pc_);
        }

        public int GetCallIndex()
        {
            if (callIndex_ >= calls_.Count)
            {
                return -1;
            }
            return callIndex_;
        }

        public int SetPC(int value)
        {
            return pc_ = value;
        }

        public int GetPC()
        {
            return pc_;
        }

        public int GetSP()
        {
            return sp_;
        }
        public int GetStackValue(int index)
        {
            return stack_[index];
        }
        public int GetStackLen() {
            return stack_.Length;
        }

        public int GetMP()
        {
            return mp_;
        }

        public string GetPort()
        {
            return Command.GetPortName(devPtr_);
        }

        public int GetMemValue(int index)
        {
            return memory_[index];
        }

        public int GetMemLen() {
            return memory_.Length;
        }

        public int[] GetMem() {
            return memory_;
        }

        public int GetRegQ()
        {
            return regQ_;
        }

        public int GetRegValue(int index)
        {
            return regCommon_[index];
        }

        public int GetF()
        {
            return f_;
        }

        public int GetY()
        {
            return y_;
        }

        public int GetPrevRegQ()
        {
            return prevRegQ_;
        }

        public int GetPrevRegA()
        {
            return prevRegA_;
        }

        public int GetPrevRegB()
        {
            return prevRegB_;
        }

        public int GetR()
        {
            return r_;
        }

        public int GetS()
        {
            return s_;
        }

        public bool GetZ()
        {
            return z_;
        }

        public bool GetF3()
        {
            return f3_;
        }

        public bool GetC4()
        {
            return c4_;
        }

        public bool GetOvr()
        {
            return ovr_;
        }

        public bool GetG()
        {
            return g_;
        }

        public bool GetP()
        {
            return p_;
        }

        public Call GetCall(int index)
        {
            return calls_[index];
        }

        public bool AddCall(int index, int code, int arg0, int arg1)
        {
            if (!mapCalls_.ContainsKey(code)) return false;
            if (arg0 > 0xff || arg1 > 0xff) return false;

            Call call = new Call(code, arg0, arg1);
            if (mapJumps_.ContainsKey(code))
            {
                switch (code)
                {
                    case 0:
                        call = new Call(code, arg0, arg1, true, JumpType.JMP);
                        break;
                    case 1:
                        call = new Call(code, arg0, arg1, true, JumpType.JC4);
                        break;
                    case 2:
                        call = new Call(code, arg0, arg1, true, JumpType.JZ);
                        break;
                    case 3:
                        call = new Call(code, arg0, arg1, true, JumpType.JSNC4);
                        break;
                    case 4:
                        call = new Call(code, arg0, arg1, true, JumpType.JNZ);
                        break;
                }
            }
            calls_.Insert(index, call);
            return true;
        }

        public bool AddCall(int index, int code, int arg0, int arg1, bool altCommandAddress, JumpType flag)
        {
            if (!mapCalls_.ContainsKey(code)) return false;
            if (arg0 > 0xff || arg1 > 0xff) return false;

            Call call = new Call(code, arg0, arg1, altCommandAddress, flag);
            calls_.Insert(index, call);
            return true;
        }
        public bool UpdateCall(int index, int code, int arg0, int arg1)
        {
            if (arg0 > 0xff || arg1 > 0xff) return false;
            if (!AddCall(index, code, arg0, arg1)) return false;
            RemoveCall(index + 1);
            return true;
        }

        public void RemoveCall(int index)
        {
            calls_.RemoveAt(index);
        }

        public void MoveCallUp(int index)
        {
            if (index <= 0)
            {
                return;
            }
            calls_.Insert(index - 1, calls_[index]);
            calls_.RemoveAt(index + 1);
        }

        public void MoveCallDown(int index)
        {
            if (index >= calls_.Count - 1)
            {
                return;
            }
            calls_.Insert(index + 2, calls_[index]);
            calls_.RemoveAt(index);
        }

        public int CallsCount()
        {
            return calls_.Count();
        }

        public Call LastCall()
        {
            return calls_.Last();
        }

        private bool CheckMapCall(int code, string name, int addr)
        {
            if (mapCalls_.ContainsKey(code)) return false;
            if (code < 0x40 && code > 0xffff) return false;

            if (name.Length > Call.NAME_MAX_SIZE) return false;
            List<Tuple<string, int>> nameAddrList = new List<Tuple<string, int>>(mapCalls_.Values);
            foreach (Tuple <string, int> item in nameAddrList)
            {
                if (name == item.Item1) return false;
            }
            
            return true;
        }

        public bool AddMapCall(int code, string name, int addr)
        {
            if (!CheckMapCall(code, name, addr)) return false;
            mapCalls_.Add(code, new Tuple<string, int>(name, addr));
            return true;
        }

        public bool RemoveMapCall(int code)
        {
            if (!mapCalls_.ContainsKey(code)) return false;
            foreach (Call item in calls_)
            {
                if (item.GetCode() == code) return false;
            }
            mapCalls_.Remove(code);
            return true;
        }

        public bool UpdateMapCall(int code, string name, int addr)
        {
            if (name.Length > Call.NAME_MAX_SIZE) return false;
            if (!mapCalls_.ContainsKey(code)) return false;
            Tuple<string, int> item = mapCalls_[code];
            item = Tuple.Create(name, addr);
            mapCalls_[code] = item;
            return true;
        }

        public Dictionary<int, Tuple<string, int>> GetMapCall()
        {
            return mapCalls_;
        }

        private void SaveAsMtemu_(FileStream fstream)
        {
            int callsSize = CallsCount() * (Call.CallSize());
            int mapCallsSize = mapCalls_.Count() * (2 * sizeof(UInt16) + Call.NAME_MAX_SIZE);
            int commandsSize = CommandsCount() * (commandSize_ + 1);
            byte[] output = new byte[fileHeader_.Length + mapCallsSize + callsSize + commandsSize + 3 * sizeof(UInt16)];

            int seek = 0;
            for (; seek < fileHeader_.Length; ++seek)
            {
                output[seek] = fileHeader_[seek];
            }

            output[seek++] = (byte)(mapCalls_.Count >> 8);
            output[seek++] = (byte)mapCalls_.Count;

            foreach (KeyValuePair<int, Tuple<string, int>> pair in mapCalls_)
            {
                output[seek++] = (byte)(pair.Key >> 8);
                output[seek++] = (byte)pair.Key;
                byte[] name = Encoding.UTF8.GetBytes(pair.Value.Item1);
                for (int c = 0; c < Call.NAME_MAX_SIZE; ++c)
                {
                    output[seek++] = (byte)(c < name.Length ? name[c] : 0);
                }
                output[seek++] = (byte)(pair.Value.Item2 >> 8);
                output[seek++] = (byte)pair.Value.Item2;
            }



            Call[] callsArr = calls_.ToArray();
            output[seek++] = (byte)(calls_.Count >> 8);
            output[seek++] = (byte)calls_.Count;

            for (int i = 0; i < callsArr.Length; ++i)
            {
                output[seek++] = (byte)(callsArr[i].GetCode() >> 8);
                output[seek++] = (byte)callsArr[i].GetCode();
                output[seek++] = (byte)(callsArr[i].GetArg0() >> 8);
                output[seek++] = (byte)callsArr[i].GetArg0();
                output[seek++] = (byte)(callsArr[i].GetArg1() >> 8);
                output[seek++] = (byte)callsArr[i].GetArg1();
                output[seek++] = (byte)(callsArr[i].GetAltCommandAddress() ? 1 : 0);
                output[seek++] = (byte)callsArr[i].GetFlag();
            }

            Command[] commandsArr = commands_.ToArray();
            output[seek++] = (byte)(commands_.Count >> 8);
            output[seek++] = (byte)commands_.Count;

            for (int i = 0; i < commandsArr.Length; ++i)
            {
                output[seek++] = (byte)(commandsArr[i].isOffset ? 1 : 0);
                for (int j = 0; j < commandSize_; ++j)
                {
                    output[seek++] = (byte)((commandsArr[i][2 * j] << 4) + commandsArr[i][2 * j + 1]);
                }
            }

            fstream.Write(output, 0, output.Length);
            fstream.SetLength(fstream.Position);
        }

        public bool SaveFile(string filename)
        {
            using (FileStream fstream = new FileStream(filename, FileMode.OpenOrCreate))
            {
                SaveAsMtemu_(fstream);
                return true;
            }
        }

        public bool OpenRaw(byte[] input)
        {
            int seek = 0;
            for (; seek < fileHeader_.Length; ++seek)
            {
                if (input[seek] != fileHeader_[seek])
                {
                    return false;
                }
            }

            mapCalls_.Clear();
            calls_.Clear();
            commands_.Clear();
            Reset();

            int mapCallCount = (input[seek++] << 8) + input[seek++];
            for (int i = 0; i < mapCallCount; ++i)
            {
                int code = (input[seek++] << 8) + input[seek++];
                byte[] byte_name = new byte[Call.NAME_MAX_SIZE];
                for (int c = 0; c < Call.NAME_MAX_SIZE; ++c)
                {
                    if (input[seek++] != 0)
                    {
                        byte_name[c] = input[seek - 1];
                    }
                }
                string name = Encoding.UTF8.GetString(byte_name);
                int addr = (input[seek++] << 8) + input[seek++];
                if (!AddMapCall(code, name, addr)) return false;
            }


            int callsCount = (input[seek++] << 8) + input[seek++];

            for (int i = 0; i < callsCount; ++i)
            {
                int code = (input[seek++] << 8) + input[seek++];
                int arg0 = (input[seek++] << 8) + input[seek++];
                int arg1 = (input[seek++] << 8) + input[seek++];
                bool altCommandAddress = (input[seek++] == 1);
                JumpType flag = (JumpType)Enum.Parse(typeof(JumpType), input[seek++].ToString());
                AddCall(i, code, arg0, arg1, altCommandAddress, flag);
            }

            int commandsCount = 0;
            commandsCount = (input[seek++] << 8) + input[seek++];
            if (seek + commandsCount * (commandSize_ + 1) > input.Length)
            {
                return false;
            }

            for (int i = 0; i < commandsCount; ++i)
            {
                bool isOffset = input[seek++] == 1;

                int[] words = new int[commandSize_ * 2];
                for (int j = 0; j < commandSize_; ++j, ++seek)
                {
                    words[2 * j] = input[seek] >> 4;
                    words[2 * j + 1] = input[seek] % 16;
                }
                commands_.Add(new Command(words));
                commands_.Last().isOffset = isOffset;
            }
            UpdateOffsets_();
            return true;
        }

        public bool OpenFile(string filename)
        {
            using (FileStream fstream = File.OpenRead(filename))
            {
                byte[] input = new byte[fstream.Length];
                fstream.Read(input, 0, input.Length);
                return OpenRaw(input);
            }
        }

        public byte[] ExportRaw()
        {
            int callsSize = CallsCount() * (sizeof(UInt16) + Call.COMMENT_MAX_SIZE);
            int commandsSize = CommandsCount() * (commandSize_ + 1);
            byte[] output = new byte[fileHeader_.Length + callsSize + commandsSize + 2 * sizeof(UInt16)];

            int seek = 0;
            for (; seek < fileHeader_.Length; ++seek) {
                output[seek] = fileHeader_[seek];
            }

            Call[] callsArr = calls_.ToArray();
            output[seek++] = (byte) (calls_.Count >> 8);
            output[seek++] = (byte) calls_.Count;

            for (int i = 0; i < callsArr.Length; ++i) {
                output[seek++] = (byte) (callsArr[i].GetAddress() >> 8);
                output[seek++] = (byte) callsArr[i].GetAddress();

                byte[] comment = Encoding.UTF8.GetBytes(callsArr[i].GetComment());
                for (int c = 0; c < Call.COMMENT_MAX_SIZE; ++c) {
                    output[seek++] = (byte) (c < comment.Length ? comment[c] : 0);
                }
            }

            Command[] commandsArr = commands_.ToArray();
            output[seek++] = (byte) (commands_.Count >> 8);
            output[seek++] = (byte) commands_.Count;

            for (int i = 0; i < commandsArr.Length; ++i) {
                output[seek++] = (byte) (commandsArr[i].isOffset ? 1 : 0);
                for (int j = 0; j < commandSize_; ++j) {
                    output[seek++] = (byte) ((commandsArr[i][2 * j] << 4) + commandsArr[i][2 * j + 1]);
                }
            }
            return output;
        }
    }
}
