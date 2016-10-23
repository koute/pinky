NES CPU Instruction Behavior Misc Tests
----------------------------------------
These tests verify miscellaneous instruction behavior.

01-abs_x_wrap
-------------
Verifies that $FFFF wraps around to 0 for
STA abs,X and LDA abs,X

2) Write wrap-around failed
3) Read wrap-around failed


02-branch_wrap
--------------
Verifies that branching past end or before beginning
of RAM wraps around

2) Branch from $FF8x to $000x
3) Branch from $000x to $FFFx


03-dummy_reads
--------------
Tests some instructions that do dummy reads before the real read/write.
Doesn't test all instructions.

Tests LDA and STA with modes (ZP,X), (ZP),Y and ABS,X
Dummy reads for the following cases are tested:

LDA ABS,X or (ZP),Y when carry is generated from low byte
STA ABS,X or (ZP),Y
ROL ABS,X always

2) Test requires $2002 mirroring every 8 bytes to $3FFA
3) LDA abs,x
4) STA abs,x
5) LDA (z),y
6) STA (z),y
7) LDA (z,x)
8) STA (z,x)
9) ROL abs
10) ROL abs,x


04-dummy_reads_apu
------------------
Tests dummy reads for (hopefully) ALL instructions which do them,
including unofficial ones. Prints opcode(s) of failed
instructions. Requires that APU implement $4015 IRQ flag reading.

2) Official opcodes failed
3) Unofficial opcodes failed


Flashes, clicks, other glitches
-------------------------------
If a test prints "passed", it passed, even if there were some flashes or
odd sounds. Only a test which prints "done" at the end requires that you
watch/listen while it runs in order to determine whether it passed. Such
tests involve things which the CPU cannot directly test.


Alternate output
----------------
Tests generally print information on screen, but also report the final
result audibly, and output text to memory, in case the PPU doesn't work
or there isn't one, as in an NSF or a NES emulator early in development.

After the tests are done, the final result is reported as a series of
beeps (see below). For NSF builds, any important diagnostic bytes are
also reported as beeps, before the final result.


Output at $6000
---------------
All text output is written starting at $6004, with a zero-byte
terminator at the end. As more text is written, the terminator is moved
forward, so an emulator can print the current text at any time.

The test status is written to $6000. $80 means the test is running, $81
means the test needs the reset button pressed, but delayed by at least
100 msec from now. $00-$7F means the test has completed and given that
result code.

To allow an emulator to know when one of these tests is running and the
data at $6000+ is valid, as opposed to some other NES program, $DE $B0
$G1 is written to $6001-$6003.


Audible output
--------------
A byte is reported as a series of tones. The code is in binary, with a
low tone for 0 and a high tone for 1, and with leading zeroes skipped.
The first tone is always a zero. A final code of 0 means passed, 1 means
failure, and 2 or higher indicates a specific reason. See the source
code of the test for more information about the meaning of a test code.
They are found after the set_test macro. For example, the cause of test
code 3 would be found in a line containing set_test 3. Examples:

	Tones         Binary  Decimal  Meaning
	- - - - - - - - - - - - - - - - - - - - 
	low              0      0      passed
	low high        01      1      failed
	low high low   010      2      error 2


NSF versions
------------
Many NSF-based tests require that the NSF player either not interrupt
the init routine with the play routine, or if it does, not interrupt the
play routine again if it hasn't returned yet. This is because many tests
need to run for a while without returning.

NSF versions also make periodic clicks to prevent the NSF player from
thinking the track is silent and thus ending the track before it's done
testing.

-- 
Shay Green <gblargg@gmail.com>
