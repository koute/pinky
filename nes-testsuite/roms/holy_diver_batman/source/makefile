#!/usr/bin/make -f
#
# Makefile for NES game
# Copyright 2011-2013 Damian Yerrick
#
# Copying and distribution of this file, with or without
# modification, are permitted in any medium without royalty
# provided the copyright notice and this notice are preserved.
# This file is offered as-is, without any warranty.
#

# These are used in the title of the NES program and the zip file.
title = holydiverbatman
version = 0.01
testconfig = testroms/M1_P128K_C128K.nes
master = hdbm-master.nes

# Space-separated list of assembly language files that make up the
# PRG ROM.  If it gets too long for one line, you can add a backslash
# (the \ character) at the end of the line and continue on the next.
objlist = \
  wrongbanks main mapper_detect loadchr wram boardletter beepcode \
  drivers mmcdrivers mmc3drivers \
  bcd pads ppuclear

AS65 = ca65
LD65 = ld65
CFLAGS65 = 
objdir = obj/nes
srcdir = src
imgdir = tilesets

#EMU := "/C/Program Files/Nintendulator/Nintendulator.exe"
EMU := fceux
# other options for EMU are start (Windows) or gnome-open (GNOME)

# Occasionally, you need to make "build tools", or programs that run
# on a PC that convert, compress, or otherwise translate PC data
# files into the format that the NES program expects.  Some people
# write their build tools in C or C++; others prefer to write them in
# Perl, PHP, or Python.  This program doesn't use any C build tools,
# but if yours does, it might include definitions of variables that
# Make uses to call a C compiler.
CC = gcc
CFLAGS = -std=gnu99 -Wall -DNDEBUG -O

# Windows needs .exe suffixed to the names of executables; UNIX does
# not.  COMSPEC will be set to the name of the shell on Windows and
# not defined on UNIX.
ifdef COMSPEC
DOTEXE=.exe
else
DOTEXE=
endif

.PHONY: run dist zip 7z

run: $(title).nes
	$(EMU) $<

$(title).nes: $(testconfig) $(master) makefile
	cp $< $@

M%.nes: $(master) tools/make_roms.py
	cd tools; ./make_roms.py; cd ..

# Rule to create or update the distribution zipfile by adding all
# files listed in zip.in.  Actually the zipfile depends on every
# single file in zip.in, but currently we use changes to the compiled
# program, makefile, and README as a heuristic for when something was
# changed.  It won't see changes to docs or tools, but usually when
# docs changes, README also changes, and when tools changes, the
# makefile changes.
dist: zip 7z
zip: $(title)-$(version).zip
7z: $(title)-bin-$(version).7z

$(title)-$(version).zip: zip.in $(title).nes README.txt $(objdir)/index.txt
	zip -9 -u $@ -@ < $<

$(title)-bin-$(version).7z: README.txt $(testconfig)
	7z a $@ README.txt testroms/*.nes testroms/8k.sav testroms/32k.sav

$(objdir)/index.txt: makefile
	echo Files produced by build tools go here, but caulk goes where? > $@

# Rules for PRG ROM

objlistntsc = $(foreach o,$(objlist),$(objdir)/$(o).o)

map.txt $(master): nes.ini $(objlistntsc)
	$(LD65) -C $^ -m map.txt -o $(master)

$(objdir)/%.o: $(srcdir)/%.s $(srcdir)/nes.h $(srcdir)/ram.h
	$(AS65) $(CFLAGS65) $< -o $@

$(objdir)/%.o: $(objdir)/%.s
	$(AS65) $(CFLAGS65) $< -o $@

# Files that depend on .incbin'd files
$(objdir)/loadchr.o: $(objdir)/font8x5.bin $(objdir)/font8x5.chr

# Rules for CHR ROM

$(objdir)/font8x5.bin: $(imgdir)/font8x5.png
	tools/cvt8x5.py $< $@

$(objdir)/%.chr: $(imgdir)/%.png
	tools/pilbmp2nes.py $< $@

$(objdir)/%16.chr: $(imgdir)/%.png
	tools/pilbmp2nes.py -H 16 $< $@

