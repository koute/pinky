; Tests sprite 0 hit timing to within 4 CPU clocks (12 PPU clocks).
; Tests time it's cleared each frame, time it's set at upper-left
; corner, time for each PPU pixel, and time for each PPU scanline.
; Depends on proper PPU frame length (less than 29781 CPU clocks).
;
; Result codes:
; 2) Sprite 0 hit was cleared too soon
; 3) Sprite 0 hit was cleared too late
; 4) Sprite 0 hit was set too soon
; 5) Sprite 0 hit was set too late
; 6) Scanlines take too few clocks
; 7) Scanlines take too many clocks
; 8) Pixels take too few clocks
; 9) Pixels take too many clocks
;
; Shay <hotpop.com@blargg> (swap to e-mail)

      .include "prefix_ppu.a"

test_name:
      .db   "SPRITE 0 HIT TIMING",0
      .code

palette:
      .db   $0f,$0f,$0f,$0f
      .db   $0f,$0f,$0f,$0f
      .db   $0f,$0f,$0f,$0f
      .db   $0f,$0f,$0f,$0f
      
      .db   $0f,$0f,$0f,$0f
      .db   $0f,$0f,$0f,$0f
      .db   $0f,$0f,$0f,$0f
      .db   $0f,$0f,$0f,$0f
      .code

begin_test:
      jsr   sync_ppu_20
      lda   #$1e        ; 6 enable obj & bg, no clipping
      sta   $2001
      nop               ; 8
      nop
      nop
      nop
      rts               ; 6

reset:
      jsr   begin_ppu_test
      jsr   load_graphics
      
      lda   #1
      jsr   fill_nametable
      
      ; Setup sprite
      jsr   wait_vbl
      lda   #0          ; clear scroll pos
      sta   $2005
      sta   $2005
      lda   #0
      sta   $2003
      lda   #0
      sta   $2004
      lda   #1
      sta   $2004
      lda   #0
      sta   $2004
      lda   #0
      sta   $2004
      lda   #$1e        ; enable obj & bg, no clipping
      sta   $2001
      
      ; Flag clear time
      
      jsr   begin_test
      
      ldy   #60         ; 29780 delay
      lda   #98         
      jsr   delay_ya3
      
      ldy   #8          ; 2267 delay
      lda   #55         
      jsr   delay_ya2

      ldx   $2002
      ldy   $2002
      
      lda   #2;) Sprite 0 hit was cleared too soon
      sta   result
      txa
      and   #$40
      jsr   error_if_eq
      
      lda   #3;) Sprite 0 hit was cleared too late
      sta   result
      tya
      and   #$40
      jsr   error_if_ne
      
      ; Scanline 0, left side
      
      jsr   wait_vbl
      lda   #0
      sta   $2003
      lda   #0
      sta   $2004
      jsr   begin_test
      
      ldy   #225        ; 2495 delay
      lda   #1          
      jsr   delay_ya3
      
      ldx   $2002
      ldy   $2002
      
      lda   #4;) Sprite 0 hit was set too soon
      sta   result
      txa
      and   #$40
      jsr   error_if_ne
      
      lda   #5;) Sprite 0 hit was set too late
      sta   result
      tya
      and   #$40
      jsr   error_if_eq
      
      ; Scanline 100, left side
      
      jsr   wait_vbl
      lda   #0
      sta   $2003
      lda   #100
      sta   $2004
      jsr   begin_test
      
      ldy   #40         ; 13861 delay
      lda   #68         
      jsr   delay_ya4
      
      ldx   $2002
      ldy   $2002
      
      lda   #6;) Scanlines take too few clocks
      sta   result
      txa
      and   #$40
      jsr   error_if_ne
      
      lda   #7;) Scanlines take too many clocks
      sta   result
      tya
      and   #$40
      jsr   error_if_eq
      
      ; Right edge of first scanline
      
      jsr   wait_vbl
      lda   #0
      sta   $2003
      lda   #0
      sta   $2004
      lda   #3
      sta   $2003
      lda   #254
      sta   $2004
      jsr   begin_test
      
      ldy   #10         ; 2578 delay
      lda   #50         
      jsr   delay_ya2
      
      ldx   $2002
      ldy   $2002
      
      lda   #8;) Pixels take too few clocks
      sta   result
      txa
      and   #$40
      jsr   error_if_ne
      
      lda   #9;) Pixels take too many clocks
      sta   result
      tya
      and   #$40
      jsr   error_if_eq
      
      jmp   tests_passed
