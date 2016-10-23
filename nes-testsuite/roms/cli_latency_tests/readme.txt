NES 6502 CLI Latency Test ROM
-----------------------------
This ROM tests the CLI and related instructions' delay in taking effect,
and related IRQ behavior. When run on a NES it gives a passing result.
The ROM runs several tests and reports the result on screen and by
beeping a number of times. See below for the meaning of failure codes
for each test.

Source code for each test is included, and most tests are clearly
divided into sections. Support code is also included, but it runs on a
custom devcart and assembler so it will require some effort to assemble.
Contact me if you'd like assistance porting them to your setup.


CLI Latency Summary
-------------------
The RTI instruction affects IRQ inhibition immediately. If an IRQ is
pending and an RTI is executed that clears the I flag, the CPU will
invoke the IRQ handler immediately after RTI finishes executing.

The CLI, SEI, and PLP instructions effectively delay changes to the I
flag until after the next instruction. For example, if an interrupt is
pending and the I flag is currently set, executing CLI will execute the
next instruction before the CPU invokes the IRQ handler. This delay only
affects inhibition, not the value of the I flag itself; CLI followed by
PHP will leave the I flag cleared in the saved status byte on the stack
(bit 2), as expected.


cli_latency.nes
---------------
Tests the delay in CLI taking effect, and some basic aspects of IRQ
handling and the APU frame IRQ (needed by the tests). It uses the APU's
frame IRQ and first verifies that it works well enough for the tests.

The later tests execute CLI followed by SEI and equivalent pairs of
instructions (CLI, PLP, where the PLP sets the I flag). These should
only allow at most one invocation of the IRQ handler, even if it doesn't
acknowledge the source of the IRQ. RTI is also tested, which behaves
differently. These tests also *don't* disable interrupts after the first
IRQ, in order to test whether a pair of instructions allows only one
interrupt or causes continuous interrupts that block the main code from
continuing.

1) Tests passed
2) RTI should not adjust return address (as RTS does)
3) APU should generate IRQ when $4017 = $00
4) Exactly one instruction after CLI should execute before IRQ is taken
5) CLI SEI should allow only one IRQ just after SEI
6) In IRQ allowed by CLI SEI, I flag should be set in saved status flags
7) CLI PLP should allow only one IRQ just after PLP
8) PLP SEI should allow only one IRQ just after SEI
9) PLP PLP should allow only one IRQ just after PLP
10) CLI RTI should not allow any IRQs
11) Unacknowledged IRQ shouldn't let any mainline code run
12) RTI RTI shouldn't let any mainline code run

-- 
Shay Green <gblargg@gmail.com>
