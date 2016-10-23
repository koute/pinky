iter = 16 ; how many samples
time = 0 ; adjusts time DMC DMA occurs
dma  = 1 ; set to 0 to disable DMA
overhead = 0 ; subtracted from time printed

.include "dma_timing.inc"

sprites = $700

main:   print_str "T+ Clocks (decimal)",newline
	jsr run_timing
.if time = 0
	check_crc $FBADA48D
.else
	check_crc $F1A58F55
.endif
	jmp tests_passed

pre_test:
	ldx #0
:       jsr update_crc
	and #$E3
	sta sprites,x
	inx
	bne :-
	setb SPRADDR,0
	rts

test:   lda #$07
	sta $4014
	sta $100
	rts

post_test:
	ldx #0
:       lda SPRDATA
	sta SPRDATA
	cmp sprites,x
	bne :+
	inx
	bne :-
	rts

:       print_str "OAM differed "
	rts
