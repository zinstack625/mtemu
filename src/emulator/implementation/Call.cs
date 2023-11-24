namespace mtemu
{
    partial class Call
    {
        int code_;
        int arg0_;
        int arg1_;
        bool altCommandAddress_;
        JumpType flag_;

        public Call(int code, int arg0, int arg1, bool altCommandAddress, JumpType flag)
        {
            code_ = code;
            arg0_ = arg0;
            arg1_ = arg1;
            altCommandAddress_ = altCommandAddress;
            flag_ = flag;
        }

        public Call(int code, int arg0, int arg1)
        {
            code_ = code;
            arg0_ = arg0;
            arg1_ = arg1;
            altCommandAddress_ = false;
            flag_ = JumpType.Unknown;
        }

        public Call(Call other)
        {
            code_ = other.code_;
            arg0_ = other.arg0_;
            arg1_ = other.arg1_;
            altCommandAddress_ = other.altCommandAddress_;
            flag_ = other.flag_;
        }

        public int GetCode()
        {
            return code_;
        }

        public void SetCode(int code)
        {
            code_ = Helpers.Mask(code, ADDRESS_SIZE_BIT);
        }

        public int GetArg0()
        {
            return arg0_;
        }

        public void SetArg0(int arg0)
        {
            arg0_ = Helpers.Mask(arg0, ARG_SIZE_BIT);
        }

        public int GetArg1()
        {
            return arg1_;
        }

        public void SetArg1(int arg1)
        {
            arg1_ = Helpers.Mask(arg1, ARG_SIZE_BIT);
        }

        public bool GetAltCommandAddress()
        {
            return altCommandAddress_;
        }

        public void SetAltCommandAddress(bool alt_command_address)
        {
            altCommandAddress_ = alt_command_address;
        }

        public JumpType GetFlag()
        {
            return flag_;
        }

        public void SetFlag(JumpType flag)
        {
            flag_ = flag;
        }
    }
}
