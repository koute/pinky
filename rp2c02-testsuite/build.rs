use std::env;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};

fn list_files< P: AsRef< Path > >( path: P ) -> io::Result< Vec< PathBuf > > {
    let mut output = Vec::new();
    for entry_opt in try!( fs::read_dir( path.as_ref() ) ) {
        let entry = try!( entry_opt );
        let metadata = try!( fs::metadata( entry.path() ) );

        if metadata.is_dir() {
            continue;
        } else {
            output.push( entry.path() )
        }
    }

    Ok( output )
}

fn main() {
    let files = list_files( "src/tests" ).unwrap();
    let mut modules: Vec< _ > = files.into_iter().flat_map( |path| {
        let filename = path.file_name().unwrap().to_string_lossy();
        if filename.starts_with( "test_" ) == false {
            None
        } else {
            Some( filename[ 0..filename.len() - 3 ].to_owned() )
        }
    }).collect();
    modules.sort();

    let out_dir: PathBuf = env::var_os( "CARGO_MANIFEST_DIR" ).unwrap().into();
    let out_filename = out_dir.join( "src" ).join( "tests" ).join( "mod.rs" );
    let mut out = Vec::new();

    writeln!( out, "// This file was AUTOGENERATED; do not edit!" ).unwrap();
    writeln!( out, "" ).unwrap();

    for module_name in &modules {
        writeln!( out, "mod {};", module_name ).unwrap();
    }
    writeln!( out, "" ).unwrap();

    for module_name in &modules {
        writeln!( out, "pub use tests::{}::*;", module_name ).unwrap();
    }
    writeln!( out, "" ).unwrap();

    writeln!( out, "#[macro_export]" ).unwrap();
    writeln!( out, "macro_rules! rp2c02_testsuite {{" ).unwrap();
    writeln!( out, "    ($interface:ident) => (" ).unwrap();
    writeln!( out, "        mod rp2c02_testsuite {{" ).unwrap();

    for module_name in &modules {
        writeln!( out, "            #[test]" ).unwrap();
        writeln!( out, "            fn {}() {{", module_name ).unwrap();
        writeln!( out, "                use super::$interface;" ).unwrap();
        writeln!( out, "                let mut ppu = $interface::new();" ).unwrap();
        writeln!( out, "                $crate::tests::{}( &mut ppu );", module_name ).unwrap();
        writeln!( out, "            }}" ).unwrap();
    }

    writeln!( out, "        }}" ).unwrap();
    writeln!( out, "    )" ).unwrap();
    writeln!( out, "}}" ).unwrap();

    if let Ok( mut fp ) = File::open( &out_filename ) {
        let mut old_contents = Vec::new();
        if let Ok( _ ) = fp.read_to_end( &mut old_contents ) {
            if old_contents == out {
                return; // Nothing changed; no point in writing to disk.
            }
        }
    }

    let mut fp = File::create( out_filename ).unwrap();
    fp.write_all( &out[..] ).unwrap();
}
