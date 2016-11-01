#!/usr/bin/env python
from __future__ import with_statement, division
import array

INES_MIRRH = 0
INES_MIRRV = 1
INES_MIRR4 = 8
INES_NTSC = 0
INES_PAL = 1
INES_DUAL_TVSYSTEM = 2
MAPPER_NROM = 0
MAPPER_MMC1 = 1
MAPPER_UNROM = 2
MAPPER_CNROM = 3
MAPPER_MMC3 = 4
MAPPER_MMC5 = 5
MAPPER_AOROM = 7
MAPPER_MMC2 = 9
MAPPER_MMC4 = 10
MAPPER_CPROM = 13
MAPPER_A53 = 28
MAPPER_BNROM = 34
MAPPER_GNROM = 66
MAPPER_FME7 = 69
MAPPER_HOLYDIVER = (78, 3)
MAPPER_MMC3_TLSROM = 118
MAPPER_MMC3_TQROM = 119
MAPPER_UNROM_CRAZY = 180
MAPPER_ACTIVE = 228

def log2(value):
    if value < 1:
        return -1
    logvalue = 0
    while value >= 0x1000000:
        value >>= 24
        logvalue += 24
    if value >= 0x10000:
        value >>= 16
        logvalue += 16
    if value >= 0x100:
        value >>= 8
        logvalue += 8
    if value >= 0x10:
        value >>= 4
        logvalue += 4
    if value >= 4:
        value >>= 2
        logvalue += 2
    return logvalue + (value >> 1)

def make_nes2_ramsize(size):
    if not 0 <= size <= 1048576:
        return ValueError("RAM size must be 0 to 1048576 bytes, not %d" % size)
    if size == 0:
        return 0
    return max(1, log2(size - 1) - 5)
    
def make_nes2_ramsizes(unbacked, backed):
    unbacked = make_nes2_ramsize(unbacked)
    backed = make_nes2_ramsize(backed)
    return unbacked | (backed << 4)

valid_mirrorings = {
    0: 0, 1: 1, 8: 8, 9: 9,
    'H': 0, 'V': 1, 'h': 0, 'v': 1, '4': 8
}

def format_memsize(size):
    if size < 1024:
        return "%d" % size
    if size < 1048576:
        return "%dK" % (size // 1024)
    return "%dM" % (size // 1048576)

def make_nes2_header(prgsize, chrsize=0, mapper=0, mirroring=INES_MIRRV,
                     prgramsize=0, chrramsize=0, tvsystem=0):
    """Make a byte string representing a 16-byte NES 2.0 header.

prgsize -- Size of PRG ROM in bytes (multiple of 16384)
chrsize -- Size of CHR ROM in bytes (multiple of 8192)
mapper -- iNES mapper number (0-4095) or a tuple (mapper, submapper)
mirroring -- iNES mirroring code (0, 1, 8, 9) or letter ('H', 'V', '4')
prgramsize -- Sizes of PRG RAM as tuple (not battery-backed, battery-backed)
chrramsize -- Sizes of CHR RAM as tuple (not battery-backed, battery-backed)

"""
    if isinstance(mapper, tuple):
        mapper, submapper = mapper
    else:
        submapper = 0
    if not isinstance(prgramsize, tuple):
        prgramsize = (prgramsize, 0)
    if not isinstance(chrramsize, tuple):
        chrramsize = (chrramsize, 0)

    if not 16384 <= prgsize < 4096 * 16384:
        raise ValueError("PRG ROM size must be 16384 to 67092480 bytes, not %d" % prgsize)
    if prgsize % 16384:
        raise ValueError("PRG ROM size must be a multiple of 16384 bytes, not %d" % prgsize)
    prgsize = prgsize // 16384
    if not 0 <= chrsize < 4096 * 8192:
        raise ValueError("CHR ROM size must be 0 to 33546240 bytes, not %d" % chrsize)
    if chrsize % 8192:
        raise ValueError("CHR ROM size must be a multiple of 8192 bytes, not %d" % chrsize)
    chrsize = chrsize // 8192
    if not 0 <= mapper < 4096:
        raise ValueError("mapper must be 0 to 4095, not %d" % mapper)
    if not 0 <= submapper < 16:
        raise ValueError("submapper must be 0 to 15, not %d" % submapper)
    try:
        mirroring = valid_mirrorings[mirroring]
    except KeyError:
        raise ValueError("mirroring must be 0, 1, 8, 9, 'H', or 'V', not %s" % mirroring)
    if tvsystem >= 4:
        raise ValueError("mirroring must be 0-3, not %d" % tvsystem)
    prgramsize = make_nes2_ramsizes(*prgramsize)
    chrramsize = make_nes2_ramsizes(*chrramsize)
    battery = 2 if ((chrramsize | prgramsize) & 0xF0) else 0

    header = array.array('B', "NES\x1a")
    header.append(prgsize & 0x0FF)
    header.append(chrsize & 0x0FF)
    header.append(mirroring | battery | ((mapper & 0x00F) << 4))
    header.append((mapper & 0x0F0) | 0x08)

    header.append(((mapper & 0xF00) >> 8) | (submapper << 4))
    header.append(((prgsize & 0xF00) >> 8) | ((chrsize & 0xF00) >> 4))
    header.append(prgramsize)
    header.append(chrramsize)
    header.append(tvsystem)
    header.extend([0] * (16 - len(header)))
    return header.tostring()

# PRG ROM size or range, CHR ROM size or range,
# mapper, mirroring, PRG RAM size(s)
romspecs_all = [
    # Discretes
    (32768, 8192,
     MAPPER_NROM, (INES_MIRRV, INES_MIRRH), (0, 2048, 4096)),
    (32768, (16384, 32768),
     MAPPER_CNROM, (INES_MIRRV, INES_MIRRH), 0),
    ((32768, 524288), 0,
     MAPPER_UNROM, (INES_MIRRV, INES_MIRRH), 0),
    ((32768, 524288), 0,
     MAPPER_UNROM_CRAZY, (INES_MIRRV, INES_MIRRH), 0),
    (32768, 0,
     MAPPER_CPROM, (INES_MIRRV, INES_MIRRH), 0),
    ((32768, 262144), 0,
     MAPPER_AOROM, 0, 0),
    ((32768, 524288), 0,
     MAPPER_BNROM, (INES_MIRRV, INES_MIRRH), 0),
    ((65536, 131072), (16384, 32768),
     MAPPER_GNROM, (INES_MIRRV, INES_MIRRH), 0),

    # SGROM SNROM SUROM SOROM SXROM
    ((32768, 524288), 0,
     MAPPER_MMC1, 0, (0, 8192, (8192, 8192), 32768)),
    # MMC1 with CHR ROM
    ((32768, 262144), (16384, 131072),
     MAPPER_MMC1, 0, (0, 8192)),
    # TKSROM TLSROM
    ((32768, 524288), (16384, 262144),
     MAPPER_MMC3_TLSROM, 0, (0, 8192)),
    # Mega Man 4/6 and TNROM
    ((32768, 524288), 0,
     MAPPER_MMC3, 0, (0, 8192)),
    # Rest of MMC3
    ((32768, 524288), (16384, 262144),
     MAPPER_MMC3, 0, (0, 8192)),
    # BTR and JxROM
    ((32768, 262144), (16384, 262144),
     MAPPER_FME7, 0, (0, 8192)),
    # Holy Diver
    ((32768, 131072), (16384, 131072),
     MAPPER_HOLYDIVER, 0, 0),
    # PNROM
    ((32768, 131072), (16384, 131072),
     MAPPER_MMC2, 0, 0),
    # FxROM
    ((32768, 131072), (16384, 131072),
     MAPPER_MMC4, 0, (0, 8192)),
]

romspecs_oneofeach = [
    # Discretes
    (32768, 8192, MAPPER_NROM, INES_MIRRV, 0),
    (32768, 32768, MAPPER_CNROM, INES_MIRRH, 0),
    (131072, 0, MAPPER_UNROM, INES_MIRRV, 0),
    (131072, 0, MAPPER_UNROM_CRAZY, INES_MIRRH, 0),
    (131072, 0, MAPPER_AOROM, 0, 0),
    (131072, 0, MAPPER_BNROM, INES_MIRRH, 0),
    (65536, 16384, MAPPER_GNROM, INES_MIRRV, 0),
    (131072, 32768, MAPPER_MMC1, 0, (0, 8192)),
    (131072, 0, MAPPER_MMC1, 0, 0),
    (524288, 0, MAPPER_MMC1, 0, ((0, 8192), (0, 32768))),
    (131072, 131072, MAPPER_MMC1, 0, (0, 8192)),
    (131072, 65536, MAPPER_MMC2, INES_MIRRV, 0),
    (262144, 262144, MAPPER_MMC3, 0, 0),
    (131072, 0, MAPPER_MMC3, 0, 0),
    (131072, 65536, MAPPER_MMC3_TLSROM, 0, 0),
    (131072, 65536, MAPPER_MMC4, INES_MIRRV, 8192),
    (131072, 65536, MAPPER_FME7, 0, 8192),
    (131072, 65536, MAPPER_HOLYDIVER, 0, 0),
    (524288, 0, MAPPER_A53, 0, 0),
]

romspecs = romspecs_oneofeach

def log_xrange(start=1, end=None, step=2):
    if end is None:
        start, end = 1, start
    while start <= end:
        yield start
        start = start * step

filename_mirroring = {0: 'H', 1: 'V', 8: '4', 9: '4V'}
switchable_mirror_mappers = {
    MAPPER_MMC1, MAPPER_MMC2, MAPPER_MMC3, MAPPER_MMC3_TQROM,
    MAPPER_MMC3_TLSROM, MAPPER_MMC4, MAPPER_AOROM, MAPPER_HOLYDIVER,
    MAPPER_FME7, MAPPER_A53
}

def handle_single_rom(prgsize, chrsize, mapper, mirror, prgramsize):
    filename = ['M%d' % (mapper[0] if isinstance(mapper, tuple) else mapper),
                '.%d' % mapper[1] if isinstance(mapper, tuple) else '',
                '_P', format_memsize(prgsize),
                '_C%s' % format_memsize(chrsize) if chrsize else '',
                '_%s' % filename_mirroring[mirror & 0x09]
                if mapper not in switchable_mirror_mappers
                else '',
                '_W%s' % format_memsize(prgramsize[0]) if prgramsize[0] else '',
                '_S%s' % format_memsize(prgramsize[1]) if prgramsize[1] else '',
                '.nes']
    filename = ''.join(filename)
    chrramsize = (8192 if mapper == MAPPER_MMC3_TQROM
                  else 0 if chrsize > 0
                  else 16384 if mapper == MAPPER_CPROM
                  else 8192)
    header = make_nes2_header(prgsize, chrsize, mapper, mirror,
                              prgramsize, chrramsize)

    # SUROM/SXROM can't guarantee PRG A18 until CHR is set up
    # so duplicate the test in all 256K outer banks
    dupli_prgsize = min(262144, prgsize)

    # Place right bank in the last bank
    prgrom = array.array('B', [0xFF]) * (dupli_prgsize - 32768)
    prgrom.fromstring(master_prgrom)

    # Place wrong bank code into all 4K banks
    wrong_bank = array.array('B', master_prgrom[0x7F80:0x8000])
    wrong_bank[-4] = 0x80  # Set reset vector
    wrong_bank[-3] = 0xFF
    for i in xrange(4096, dupli_prgsize, 4096):
        prgrom[i - 128:i] = wrong_bank
    del wrong_bank

    # Emulators commonly boot AOROM, BNROM, GNROM, and
    # UNROM (Crazy Climber) to the first bank.  There's a stub
    # in $BF6C that tries switching to the last bank.
    # Put it in all 16K banks.
    gnromstub = array.array('B', master_prgrom[0x3F6C:0x3F80])
    for i in xrange(0, dupli_prgsize, 16384):
        prgrom[i + 0x3F6C:i + 0x3F80] = gnromstub
    for i in xrange(0, dupli_prgsize - 16384, 16384):
        prgrom[i + 0x3FFC] = 0x6C
        prgrom[i + 0x3FFD] = 0xBF

    # SUROM/SXROM duplication
    prgrom = prgrom * (prgsize // len(prgrom))

    # Place right bank in first 16K (for #180 UNROM Crazy)
    prgrom[:0x3F6C] = array.array('B', master_prgrom[:0x3F6C])

    # Finally, add bank numbers to PRG ROM
    for i in xrange(4096 - 8, prgsize, 4096):
        prgrom[i] = i // 4096
        prgrom[i + 1] = 0
    prgrom[-7] = 1

    # Add bank numbers to CHR ROM
    chrrom = array.array('B', master_chrrom) * (chrsize // len(master_chrrom))
    for i in xrange(508, chrsize, 1024):
        chrrom[i] = i // 1024

    rom = ''.join([header, prgrom.tostring(), chrrom.tostring()])
    return (filename, rom)

def expand_romspec(prgsizes, chrsizes, mapper, mirrors, ramsizes):
    from collections import Sequence
    from itertools import product

    if not isinstance(prgsizes, Sequence):
        prgsizes = (prgsizes, prgsizes)
    if not isinstance(chrsizes, Sequence):
        chrsizes = (chrsizes, chrsizes)
    if not isinstance(mirrors, Sequence):
        mirrors = (mirrors,)
    if not isinstance(ramsizes, Sequence):
        ramsizes = (ramsizes,)
    ramsizes = (spec
                for sz in ramsizes
                for spec in (set([(sz, 0), (0, sz)])
                             if not isinstance(sz, Sequence)
                             else (sz,)))
    
    return product(log_xrange(*prgsizes),
                   log_xrange(*chrsizes) if chrsizes[0] else (0,),
                   (mapper,), mirrors, ramsizes)

test_rom_folder = '../testroms'
master_file = '../hdbm-master.nes'

def main():
    import os
    from itertools import starmap  # starmap(f, rows = f(*row) for row in rows)
    global master_prgrom, master_chrrom
    with open(master_file, 'rb') as infp:
        infp.read(16)
        master_prgrom = infp.read(32768)
        master_chrrom = infp.read(8192)
        
##    h = make_nes2_header(262144, 131072, MAPPER_MMC3, INES_MIRRV,
##                         (0, 8192), 0, INES_NTSC)
##    print " ".join("%02x" % ord(c) for c in h)
    specs = [single_rom
             for romspec in romspecs
             for single_rom in expand_romspec(*romspec)]
    print len(specs), "specs"
    roms = starmap(handle_single_rom, specs)
    total_sz = 0

    if not os.path.isdir(test_rom_folder):
        os.mkdir(test_rom_folder)
    seen_names = set()
    for (filename, rom) in roms:
        if filename in seen_names:
            print "duplicate filename", filename
        seen_names.add(filename)
        total_sz += len(rom)
        with open(os.path.join(test_rom_folder, filename), 'wb') as outfp:
            outfp.write(rom)
    print "total: %s" % format_memsize(total_sz)

if __name__=='__main__':
    main()
