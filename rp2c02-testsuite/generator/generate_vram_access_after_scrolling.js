"use strict";

var common = require( "./common" );
var ppumask = common.ppumask;
var ppuctrl = common.ppuctrl;
var sprite = common.sprite;

common.generate( "vram_access_after_scrolling", function( ctx ) {
    ctx.fill_vram_with_test_data();

    var ppuctrl_value = ppuctrl()
        .set_base_background_tilemap( 1 )
        .set_use_second_pattern_table_for_background( true )
        .set_double_height_sprite_mode( true )
        .set_should_generate_vblank_nmi( true );

    ctx.cpu_write_and_step( ppuctrl_value );

    var ppumask_value = ppumask()
        .set_show_background_in_leftmost_8_pixels( true )
        .set_show_sprites_in_leftmost_8_pixels( true )
        .set_show_background( true )
        .set_show_sprites( true );

    ctx.cpu_write_and_step( ppumask_value );

    ctx.repeat( 255, ctx.step_pixel );

    var test = function() {
        ctx.test.temporary_address();
        ctx.test.current_address();
        ctx.test.next_vram_read();
        ctx.step_pixel();
    };

    ctx.test.temporary_address();
    ctx.test.current_address();
    ctx.cpu_write_and_step( 0x2005, 0x29 );
    ctx.test.temporary_address();
    ctx.test.current_address();
    ctx.cpu_write_and_step( 0x2005, 0x00 );
    ctx.repeat( 93, test );

    ctx.repeat( 257, test );

    ctx.test.cpu_read( 0x2002 );
    ctx.cpu_write_and_step( 0x2005, 0x00 );
    ctx.cpu_write_and_step( 0x2005, 0x29 );
    ctx.step_scanline();

    ctx.repeat( 257, test );
});
