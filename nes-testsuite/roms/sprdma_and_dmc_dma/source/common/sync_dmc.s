.align 64

; Synchronizes to DMC timer within 7 cycles.
; Preserved: A, X, Y
; Time: ~1000 cycles max
sync_dmc_fast:
	pha
	
	lda #0
	sta $4013
	lda #$0F
	sta $4010
	sta SNDCHN
	
	; Start twice (first will clear immediately)
	lda #$1F
	sta SNDCHN
	nop
	sta SNDCHN
	
	lda #$10
:       bit SNDCHN
	bne :-
	
	pla
	rts


; Synchronizes precisely with DMC timer. Leaves DMC at
; maximum rate, length 0 (1 byte). To avoid stopping
; other sound channels when writing to SNDCHN, it
; writes $1F rather than $10.
; Preserved: A, X, Y
; Time: ~680-3800 cycles
sync_dmc:
	; This coarse synchronization ensures that fine loop
	; is only a few cycles away, rather than potentially
	; thousands, which would take that many iterations
	; (*414 cycles each) to synchronize.
	jsr sync_dmc_fast
	
	pha
	
	; DO NOT write to memory. It affects timing.
	
	nop             ; 6 fine-tune: 6=OK 7=fail
	nop
	nop
	lda #55         ; 386 delay for first iter
	bne :+          ; 3
	
	; Fine synchronize. 433 cycles per iter.
	; DMC sample takes 432 cycles, thus this
	; loop will slowly creep up on it until it
	; completes just before the final bit SNDCHN
	; rather than after as all previous iterations.
@sync:  lda #59         ; 414 delay
:       sec
	sbc #1
	bne :-
			; 4 DMC wait-states
	lda #$1F        ; 2
	sta SNDCHN      ; 4
	lda #$10        ; 2
	bit SNDCHN      ; 4
	bne @sync       ; 3
	
	pla
	rts
