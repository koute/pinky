#
# Linker script for Holy Diver, Batman!, a mapper test
# Copyright 2013 Damian Yerrick
#
# Copying and distribution of this file, with or without
# modification, are permitted in any medium without royalty
# provided the copyright notice and this notice are preserved.
# This file is offered as-is, without any warranty.
#
MEMORY {
  ZP:     start = $10, size = $E0, type = rw;
  # use first $10 zeropage locations as locals
  HEADER: start = 0, size = $0010, type = ro, file = %O, fill=yes, fillval=$00;
  RAM:    start = $0300, size = $0100, type = rw;
  RAM0:   start = $0400, size = $0400, type = rw;

  # individual drivers
  RAM1:   start = $0400, size = $0300, type = rw;
  RAM2:   start = $0400, size = $0300, type = rw;
  RAM3:   start = $0400, size = $0300, type = rw;
  RAM4:   start = $0400, size = $0300, type = rw;
  RAM5:   start = $0400, size = $0300, type = rw;
  RAM6:   start = $0400, size = $0300, type = rw;
  RAM7:   start = $0400, size = $0300, type = rw;
  RAM8:   start = $0400, size = $0300, type = rw;

  # The ROM is assumed to be divided into 4K banks.  The ROMxL memory
  # areas ensure that the last 128 bytes of each 4K bank are unused
  # because the post-processing builder copies the wrong-banks code
  # and bank tag there.
  ROM8:   start=$8000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROM8L:  start=$8F80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROM9:   start=$9000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROM9L:  start=$9F80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMA:   start=$A000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMAL:  start=$AF80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMB:   start=$B000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMBL:  start=$BF80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMC:   start=$C000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMCL:  start=$CF80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMD:   start=$D000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMDL:  start=$DF80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROME:   start=$E000, size=$0F80, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMEL:  start=$EF80, size=$0080, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMF:   start=$F000, size=$0F6C, type=ro, file=%O, fill=yes, fillval=$FF;
  ROMFL:  start=$FF6C, size=$0094, type=ro, file=%O, fill=yes, fillval=$FF;
  CHRROM: start=$0000, size=$2000, type=ro, file=%O, fill=yes, fillval=$FF;
}

SEGMENTS {
  INESHDR:  load = HEADER, type = ro, align = $10;
  ZEROPAGE: load = ZP, type = zp;
  BSS:      load = RAM, type = bss, define = yes, align = $100;

  # The amount of memory depends on the level of confidence in the
  # mapper.  At boot, only the last 4K can be relied on.
  LOWCODE:   load = ROMF, run = RAM0, type = ro, define = yes;
  STARTUP:   load = ROMF, type = ro;
  WRONGBANK: load = ROMFL, type = ro, start = $FF80;

  # After the first round of mapper detection, $C000-$FFFF is
  # available.  For MMC2, MMC4, UNROM, and Holy Diver, this region
  # is always available.
  CODE:     load = ROMC, type = ro, align = $100;
  RODATA:   load = ROMD, type = ro, align = $100;
  
  # MMC3 and FME-7 fix $E000-$FFFF even if $C000-$DFFF is swapped
  # out.  Their tests can safely run from $E000-$FFFF.
  ECODE:    load = ROME, type = ro, align = $100, optional=yes;

  # Drivers of ROMs with 32K or fixed-$8000 bankswitching can be
  # stored below the $C000 watermark
  DRIVER_GNROM: load=ROMB, run=RAM1, type=ro, define=yes;
  DRIVER_BNROM: load=ROMB, run=RAM2, type=ro, define=yes;
  GNROMSTUB:    load=ROMB, type=ro, start=$BF6C;

  # These drivers are bigger
  DRIVER_A53: load=ROMD, run=RAM3, type=ro, define=yes;
  DRIVER_MMC1: load=ROMD, run=RAM4, type=ro, define=yes;
  DRIVER_MMC2: load=ROMD, run=RAM5, type=ro, define=yes;
  DRIVER_MMC3: load=ROMD, run=RAM6, type=ro, define=yes;
  DRIVER_FME7: load=ROMD, run=RAM7, type=ro, define=yes;

  # Both fixed-$8000 and fixed-$C000 UNROM use this driver
  DRIVER_UNROM: load=ROMD, run=RAM8, type=ro, define=yes;

  CHR:      load = CHRROM, type = ro, align = $10;
}

FILES {
  %O: format = bin;
}

