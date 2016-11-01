Holy Diver Batman
an NES demo by Damian Yerrick

This demo detects what mapper it's running on through mirroring,
tests how big of a ROM it was written on, verifies that the mapper
is mostly working, and displays the result.

== The name ==

Paul Molloy of infiniteneslives.com is selling INL-ROM circuit
boards for making NES game cartridges.  These boards are configured
at manufacturing time to support the mapper circuits required by
several games.  Among them are Holy Diver by IREM and Batman: Return
of the Joker by Sunsoft.  Putting those titles together results in
something Burt Ward as Dick "Robin" Grayson might say:
"Holy diver, Batman!"

http://holysmokesbatman.com/

== How to build ==

You first need to install these:

* Python 2.6 or 2.7
* Python Imaging Library
* ca65 and ld65 (part of the cc65 compiler distribution)
* GNU Coreutils
* GNU Make

To get these on Debian or Ubuntu, you need to install the
build-essential and python-imaging packages and then build cc65 from
source.  (If you're keeping your system strictly DFSG-free, you can
delete the non-free cc65 compiler.)  To get GNU Coreutils and Make
under Windows, install MSYS from mingw.org or devkitpro.org.

The makefile performs four steps:

1. Convert the system font from PNG format to an NES-friendly format
2. Assemble all source code files
3. Link source code files into hdbm-master.nes
4. Generate ROMs with several sizes and mappers

Then you can use any PRG/CHR splitting program.  The contents of all
PRG ROMs are identical for a given size no matter the mapper, as are
all CHR ROMs.  Go ahead and burn an EPROM for each size and plug it
into your socketed boards.

== Supported boards ==

001   SxROM
      Bionic Commando, Final Fantasy, Journey to Silius
002   UNROM, UOROM (7432)
      Battle Kid, Contra, DuckTales
004   TxROM
      Super Mario Bros. 2-3, Mega Man 3-6
007   AxROM
      Battletoads, Jeopardy!
009   PNROM
      Punch-Out!!
010   FJROM, FKROM
      Famicom Wars, Fire Emblem
028   INL-ROM (Action 53)
      STREEMERZ: Action 53 Function 16 Volume One
034   BNROM
      Deadly Towers, homemade Action 53 collections
066   NROM, CNROM, GNROM
      Balloon Fight, Pipe Dream, Gumshoe
069   JLROM, JSROM
      Batman: Return of the Joker, Gimmick!, Hebereke
078.3 IF-12
      Holy Diver
118   TxSROM
      NES Play Action Football, Wanderers from Ys
180   UNROM (7408)
      Crazy Climber

== The tests ==

To understand the tests, one must first understand bank tags.
These are increasing numbers spaced at regular intervals in memory
that let the program know what bank has been switched in.  Bank tags
are placed at offset 4088 ($FF8) of every 4 KiB bank of PRG ROM and
at offset 508 ($1FC) of every 2 KiB bank of CHR ROM or RAM.

The first thing it does is copy test code into RAM that detects the
mapper by how it responds to writes to the supported boards'
mirroring ports.  Then, because a few mappers react the same way,
such as all boards with fixed mirroring, it narrows the field through
very basic bank switching tests.  This needs to be done in RAM
because 32K bank switching may cause the startup ROM bank to be
switched out.

After the mapper number is determined, it copies a driver for that
mapper into RAM.  The driver is responsible for changing 8K CHR banks
and WRAM banks and doing detailed tests.

-- CHR test --

The CHR test starts by asking the driver to switch the first 8K of
CHR memory into PPU $0000-$1FFF.  If data written to $0000 in CHR
space sticks there, it's CHR ROM.  Otherwise, it's CHR RAM, and the
test writes bank tags in CHR RAM banks 31 down to 0.It reads bank
tags from the last CHR bank to verify that they're internally
consistent.  If not, it beeps "CBT" in Morse.  If so, it knows the
size of CHR memory.

If CHR ROM is present, the test checks the font in $0000-$03FF
against a copy of the font in PRG ROM to ensure that CHR ROM can be
read.  (A mismatch produces Morse "FON".)  Then it verifies all bank
tags and saves the CHR test result for later.

If it's CHR RAM, it writes a pseudorandom pattern to all of CHR RAM,
reads it back, and saves whether it matched for later.  Then it
repeats the test seven more times with the pattern shifted by a byte
each time.  While this is going on, it buzzes the speaker to let the
user know it hasn't frozen.  Finally, it puts the bank tags back and
loads the small font into CHR RAM.

-- WRAM test --

Work RAM (WRAM), or PRG RAM, is RAM on the cartridge at CPU
$6000-$7FFF.  First it looks for the string "SAVEDATA" at $6100 and
sets a flag if it was found.  Then the same test is done as for CHR
RAM:  determine how much, then test each byte.  Finally, "SAVEDATA"
is written back.  This way, the user can test for battery backup by
running the test, powering off, and running the test again.
Pressing the Reset button will cause an incorrect result.

-- Detailed test --

Some mappers are more flexible than others in how they map PRG and
CHR banks into windows within the CPU and PPU address space.  This
test steps through all the PRG and CHR bank numbers in each window
with various combinations of banking modes.  It also checks whether
WRAM can be disabled for power-off protection (on mappers that claim
to support this) and whether the IRQ works roughly as expected.
This is no substitute for an exhaustive mapper-specific test, but
it should help determine whether the chips are soldered properly.

== Morse codes ==

If a test fails hard, your NES may beep Morse code at you.
Find a ham and fix your soldering.

WB  Wrong bank at startup.  INL's versions of the ASIC mappers
    guarantee that the LAST 4 KiB of the cart is swapped into
    $F000-$FFFF at power on.  Discrete may not be so lucky.
RB  Attempt to return to the last bank after a test failed.
MIR The nametable mirroring for this mapper doesn't match any of the
    supported mappers.  Check PA13-PA10, /PA13, CIRAM A10, and CIRAM
    enable, and don't try running the 78.3 test on an emulator that
    does not support NES 2.0 format.
DRV The driver for the mapper is missing.
FON Font failed to verify.  The CHR ROM or RAM is bad.
CBT CHR bank tags failed to verify.

The mapper 34 test may freeze in emulators that do not support the
NES 2.0 format.  They make both the BNROM registers and the NINA-001
registers available at the same time, when the current best practice
is to switch between the two based on CHR size.

== Displayed result ==

After the buzzing ends, the following variables are valid:

cur_mapper
  iNES mapper number of the detected mapper.
last_prg_bank
  Size of PRG ROM in 4096 byte units, minus one
is_chrrom
  Zero if CHR RAM is present; nonzero if CHR ROM is present
last_chr_bank
  Size of CHR ROM or CHR RAM in 8192 byte units, minus one
chr_test_result
  Zero if bank tags were read correctly from CHR ROM or if all
  bytes of CHR RAM correctly hold values
has_wram
  Nonzero if $6000-$7FFF is RAM
has_savedata
  Nonzero if "SAVEDATA" was in $6100 of the first PRG RAM bank
  at startup
last_wram_bank
  Size of PRG RAM in 8192 byte units, minus one; undefined if
  has_wram is zero
wram_test_result
  Zero if all bytes of PRG RAM correctly hold values; undefined
  if has_wram is zero
driver_prg_result
  Detailed PRG RAM and PRG ROM test results
driver_chr_result
  Detailed IRQ and CHR test results

All this information is displayed on the screen.  The detailed code
is 4 digits: WRAM, PRG ROM, IRQ, and CHR ROM/RAM.  Zero is normal;
anything else reflects something unexpected.

Mpr Code  Meaning
001 1xxx  $E000 bit 4 does not disable WRAM
001 4xxx  $A000 bit 4 does not disable WRAM (SNROM)
          $A000 bit 4 disables WRAM (all but SNROM)
004 2xxx  Read-only mode not present

In iNES format environments that don't support NES 2.0, such as FCEUX
and PowerPak, the MMC3 test will return a warning about lack of write
protection on WRAM.  These environments don't implement this feature
because not implementing it gets MMC6 games (StarTropics 1 and 2) to
run with the MMC3 driver.

Some INL-ROM products implement only a subset of the entire mapper as
a cost-saving measure.  For example, MMC3 may lack WRAM protection or
may fix C and P values.  Detailed codes will reflect any unimplemented
features.

For the convenience of users testing large numbers of boards without
looking at the screen, the result is also beeped through the speaker
with one note per nibble so that an attentive ear can pick up any
deviation from the intended melody.

    0: Bb       4: F        8: Mid C    C: G
    1: Low C    5: G        9: D        D: A
    2: D        6: A        A: E        E: B
    3: Eb       7: Bb       B: F        F: High C

There are three groups of nibbles:

1. Tweet, mapper number (2)
2. Tweet, PRG ROM size in 32768 byte units (2), PRG RAM size in
   8192 byte units (2), buzz if PRG RAM defective, ding if battery,
   PRG RAM detailed result (1), PRG ROM detailed result (1)
3. Tweet, CHR size in 8192 byte units minus 1 (2), CHR RAM or
   ROM (1), buzz if CHR RAM or ROM defective, IRQ detailed
   result (1), CHR detailed result (1)

== Limits ==

Not all mappers are supported.

NROM and CNROM are detected as mapper 66, but the board name
should be correct.

The IRQ test measures only gross behavior of the MMC3 scanline
counter or the FME-7 CPU cycle counter.  It is not intended as a
substitute for mapper-specific detailed tests to ensure exact timing
of a CPLD reimplementation compared to the authentic mapper IC.

The "Flash ID" results mean nothing.  They are reserved for
future expansion.

== Possible future changes ==

* Code review to comment what is unclear
* Add other mappers as they are requested
* Port CHR conversion and ROM expansion to Python 3 and Pillow

== Legal ==

Copyright 2013 Damian Yerrick
Copying and distribution of this file, with or without
modification, are permitted in any medium without royalty provided
the copyright notice and this notice are preserved in all source
code copies.  This file is offered as-is, without any warranty.

Not sponsored or endorsed by DC Comics, IREM, Nintendo, or Sunsoft.
