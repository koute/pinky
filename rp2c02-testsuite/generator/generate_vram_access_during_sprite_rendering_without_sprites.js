"use strict";

var _ = require( "./phantom2c02/lib/lodash" );
var common = require( "./common" );
var ppumask = common.ppumask;
var sprite = common.sprite;

common.generate( "vram_access_during_sprite_rendering_without_sprites", function( ctx ) {
    var ppumask_value = ppumask()
        .set_show_sprites( true )
        .set_show_sprites_in_leftmost_8_pixels( true );

    ctx.fill_vram_with_test_data();

    ctx.cpu_write_and_step( ppumask_value );
    ctx.step_scanline();

    var test = function() {
        ctx.repeat( 257, ctx.step_pixel ); // Skip sprite evaluation.
        ctx.test.secondary_oam_contents(); // This should contain [0x00, 0xFF, 0xFF, 0xFF, ...].

        // In this test the OAM is filled with zeros. The PPU fills the secondary OAM
        // with 0xFF during dots 001..064; then during 065..256 it evaluates the OAM
        // and searches for sprites to copy to the secondary OAM. One of the quirks
        // of the PPU is that it *always* copies the first byte from a given sprite
        // slot from OAM to the secondary OAM when evaluating it, so if OAM is filled
        // with zeros and we haven't found any sprites to render on the next scanline
        // the secondary OAM will be filled with [0x00, 0xFF, 0xFF, 0xFF, ...].

        ctx.repeat( 64, function() {
            ctx.test.next_vram_read();
            ctx.step_pixel();
        });
        ctx.step_scanline();
    };

    ctx.repeat( 24, test );
});
