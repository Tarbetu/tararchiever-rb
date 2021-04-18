use rutie::{module, methods, Module, Symbol, Class, Object, VM, Integer, NilClass};
use crate::{Compressor, Decompressor};
use std::path::PathBuf;
use crate::CompressionType::*;

module!(Archiver);

methods!(
    Archiver,
    rtself,

    fn compress(from: Symbol, target: Symbol, file_name: Symbol, comp_type: Symbol, comp_level: Integer) -> NilClass {
        //Checking arguments
        if from.is_err() || target.is_err() {
            VM::raise(
                Class::from_existing("ArgumentError"), 
                "Error while parsing the paths"
            )
        }
        let from = from.unwrap().to_string();
        let target = target.unwrap().to_string();
        let comp_level = comp_level.unwrap_or(Integer::from(3)).to_u64();
        let comp_type = comp_type.unwrap_or(Symbol::new("gzip")).to_string();
        let comp_type = crate::CompressionType::new(&comp_type); 
        if let Err(e) = &comp_type {
           VM::raise(
               Class::from_existing("ArgumentError"),
               &format!("{}",e)
            )
        }
        let comp_type = comp_type.unwrap_or(GZIP);
        let file_name = if file_name.is_err() {
            match &comp_type {
                GZIP => String::from("archive.tar.gz"),
                ZSTD => String::from("archive.tar.zst"),
                XZ   => String::from("archive.tar.xz"),
                LZ4  => String::from("archive.tar.lz4"),
            } 
        } else {
            file_name.unwrap().to_string()
        };
        let comp = Compressor::new(
            PathBuf::from(from),
            PathBuf::from(target),
            file_name,
            comp_type,
            comp_level as u32,
        ); 
        if let Err(x) = &comp {
            VM::raise(
                Class::from_existing("ArgumentError"),
                &format!("{}",x)
            )
        }
        if let Err(x) = comp.unwrap().compress() {
            VM::raise(
                Class::from_existing("IOError"),
                &format!("{}",x)
            )
        }
        NilClass::new()
    }
    fn decompress(source: Symbol, target_dir: Symbol, comp_type: Symbol) -> NilClass {
        if source.is_err() || target_dir.is_err() {
            VM::raise(
                Class::from_existing("ArgumentError"), 
                "Error while parsing the paths"
            )
        }
        let source     = PathBuf::from(source.unwrap().to_str());
        let target_dir = PathBuf::from(target_dir.unwrap().to_str());
        let comp_type  = comp_type.unwrap_or(Symbol::new("gzip")).to_string();
        let comp_type  = crate::CompressionType::new(&comp_type);
        if let Err(e) = &comp_type { 
                VM::raise(
                    Class::from_existing("ArgumentError"),
                    &format!("{}",e)
                );
        }
        let comp_type = comp_type.unwrap();
        let decomp = Decompressor::new(
            source,
            target_dir,
            comp_type
        );
        if let Err(x) = decomp.decompress() {
            VM::raise(
                Class::from_existing("IOError"),
                &format!("{}",x)
            )
        }
        NilClass::new()
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn initialize_rust_lib() {
    Module::new("Archiver").define(|module| {
        module.def_self("compress_dir",compress);
        module.def_self("decompress_tar",decompress);
    });
}
