use std::fs::File;
use tar::Archive;
use std::io;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::fmt;

mod error;
use error::CompressionError;

mod ruby_binding;

use flate2::write::GzEncoder;
use xz2::write::XzEncoder;
use zstd::stream::Encoder as ZstdEncoder;

use flate2::read::GzDecoder;
use zstd::stream::Decoder as ZstdDecoder;
use lz4::Decoder as Lz4Decoder;
use xz2::read::XzDecoder;

#[derive(PartialEq, Copy, Clone)]
enum CompressionType {
    LZ4,
    GZIP,
    ZSTD,
    XZ
}

impl fmt::Display for CompressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self {
            Self::LZ4 => "lz4",
            Self::GZIP => "gzip",
            Self::ZSTD => "zstd",
            Self::XZ => "xz"
        };
        write!(f,"{}",name)
    }
}

impl CompressionType {
    fn new(symbol: &str) -> Result<CompressionType,CompressionError> {
       match &*symbol.to_lowercase() {
           "lz4" => Ok(CompressionType::LZ4),
           "gzip" => Ok(CompressionType::GZIP),
           "zstd" => Ok(CompressionType::ZSTD),
           "xz" => Ok(CompressionType::XZ),
           _ => Err(CompressionError::new(error::ErrorKind::UnknownType))
       }
    }
}

struct Compressor<T: AsRef<OsStr>> {
    from:       T,
    target:     T,
    file_name:  String,
    comp_type:  CompressionType,
    comp_level: u32
}

impl<T: AsRef<OsStr>> fmt::Display for Compressor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source: &str = self.from.as_ref().to_str().unwrap_or("Invalid Source");
        let target: &str = self.target.as_ref().to_str().unwrap_or("Invalid Target");
        write!(f,"Source: {}
        File name: {}
        Target: {}
        Type: {}
        Level: {}",source,self.file_name,target,self.comp_type, self.comp_level)
    }
}

impl<T: AsRef<Path> + AsRef<OsStr>> Compressor<T> {
    pub fn new(from: T, target: T, file_name: String, comp_type: CompressionType, comp_level: u32) -> Result<Compressor<T>, CompressionError> { 
        if (comp_type != CompressionType::ZSTD && comp_level > 9) || comp_level>21 {
           return Err(CompressionError::new(error::ErrorKind::InvalidLevel)); 
        }
        Ok(Self {
            from,
            target,
            file_name,
            comp_type,
            comp_level
        })
    }
    pub fn compress(&self) -> Result<(),CompressionError> {
        let work = self.do_it();
        if let Err(x) = work {
            return Err(CompressionError::new( error::ErrorKind::Other( x.to_string() ) ) );
        }
        Ok(())
    }
    fn do_it(&self) -> io::Result<()> {
        use CompressionType::*;
        let target_file = PathBuf::from(&self.target).join(&self.file_name);
        let target_file = File::create(target_file)?;
        //This kind of repetition worries me. 
        //If you think there is an another way, please contact me.
        match self.comp_type {
            GZIP => {
                let mut enc = GzEncoder::new(
                    target_file, 
                    flate2::Compression::new(self.comp_level)
                );
                {
                    let mut tar = tar::Builder::new(&mut enc);
                    tar.append_dir_all(".",&self.from)?;
                }
                enc.finish()?;
            },
            ZSTD => {
                let mut enc = ZstdEncoder::new(target_file, self.comp_level as i32)?;
                {
                    let mut tar = tar::Builder::new(&mut enc);
                    tar.append_dir_all(".", &self.from)?;
                }
                enc.finish()?;
            },
            LZ4 => {
                let mut enc = lz4::EncoderBuilder::new()
                                             .level(self.comp_level)
                                             .build(target_file)?;
                {
                    let mut tar = tar::Builder::new(&mut enc);
                    tar.append_dir_all(".", &self.from)?;
                }
                enc.finish().1?;
            },
            XZ => {
                let mut enc = XzEncoder::new(target_file, self.comp_level);
                {
                    let mut tar = tar::Builder::new(&mut enc);
                    tar.append_dir_all(".", &self.from)?;
                }
                enc.finish()?;
            }

        };
        Ok(())
    }
}

struct Decompressor<T: AsRef<OsStr>> {
    source: T,
    target_dir: T,
    comp_type: CompressionType
}

impl<T: AsRef<Path> + AsRef<OsStr>> Decompressor<T> {
    pub fn new(source: T, target_dir: T, comp_type: CompressionType) -> Decompressor<T> {
        Self {
            source,
            target_dir,
            comp_type
        }
    }
    fn do_it(&self) -> io::Result<()> {
        use CompressionType::*;
        let tar_file = File::open(&self.source)?;
        match &self.comp_type {
            GZIP => {
                let dnc = GzDecoder::new(tar_file);
                let mut arch = Archive::new(dnc);
                arch.unpack(&self.target_dir)?;
            },
            ZSTD => {
                let dnc = ZstdDecoder::new(tar_file)?;
                let mut arch = Archive::new(dnc);
                arch.unpack(&self.target_dir)?;
            },
            LZ4 => {
                let dnc = Lz4Decoder::new(tar_file)?;
                let mut arch = Archive::new(dnc);
                arch.unpack(&self.target_dir)?;
            },
            XZ => {
                let dnc = XzDecoder::new(tar_file);
                let mut arch = Archive::new(dnc);
                arch.unpack(&self.target_dir)?
            }
        }
        Ok(())
    }
    pub fn decompress(&self) -> Result<(), CompressionError> {
        let work = self.do_it();
        if let Err(x) = work {
            return Err(CompressionError::new( error::ErrorKind::Other( x.to_string() ) ) );
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::fs::{create_dir, write};
    use std::env::temp_dir;
    use super::*;

    fn env_dir<T: fmt::Display>(dir_name: T) -> PathBuf {
        Path::new(&temp_dir()).join(format!(".tararchiver_test_{}",dir_name))
    }

    fn prepare_the_environment<T: fmt::Display>(dir_name: T) -> io::Result<PathBuf> {
        let env_dir = env_dir(dir_name);
        if env_dir.exists() {
            std::fs::remove_dir_all(&env_dir)?;
        }
        create_dir(&env_dir).unwrap();
        let readme_txt = Path::new(&env_dir).join("README.txt");
        write(readme_txt, "
        This directory just for testing the 'Tararchiver' gem. 
        If you think this directory take up an unnecessary space,
        Don't hesitate to delete this directory
        if you sure about Cargo's* testing is over.
        *Cargo: Cargo is a tool for developing Rust programs.
            ")?;
        Ok(env_dir)
    }
    fn prepare_the_data_to_be_compress(path: &PathBuf) -> PathBuf {
        //Creating a data to be compressed
        let mut path = PathBuf::from(&path);
        path.push("for_compressing");
        create_dir(&path).unwrap();
        path.push("data.txt");
        let mut written_data: String;
        write(&path, "Hi! Let guess my lucky number!").unwrap();
        for i in 1..8888 {
            written_data = std::fs::read_to_string(&path).unwrap();
            write(&path, format!("{} \n This might be my lucky number: {}\n",written_data,i)).unwrap();
         }
        path
    }
    fn clean_up(env_dir: PathBuf) -> io::Result<()> {
        eprintln!("Cleaning up the testing directory: {}",env_dir.as_path().display());
        std::fs::remove_dir_all(env_dir)?;
        Ok(())
    }
    fn compress_with(env: &PathBuf, comp_type: CompressionType) {
        let source = prepare_the_data_to_be_compress(env);
        let comp = Compressor::new(
            source.parent().unwrap(), 
            &env,
            "dummy_comp.tar.generic".to_string(),
            comp_type,
            3
        );
        let comp = comp.unwrap();
        comp.compress().unwrap();
        assert!({
            let mut tar_gz = PathBuf::from(&env);
            tar_gz.push("dummy_comp.tar.generic");
            tar_gz.exists()
        })
    }
    fn decompress_a_tar(env: &PathBuf, comp_type: CompressionType) {
        let source = Path::new(&env).join("dummy_comp.tar.generic");
        let target = Path::new(&env).join("target_dir");
        create_dir(&target).unwrap();
        let comp = Decompressor::new(&source, &target, comp_type);
        comp.decompress().unwrap();
        assert!(&target.exists());
    }
    fn compress_and_decomress(comp_type: CompressionType) {
        let env = prepare_the_environment(comp_type).expect(&format!("Error while creating the working directory for {} testing",comp_type));
        compress_with(&env, comp_type);
        decompress_a_tar(&env, comp_type);
        clean_up(env).unwrap();
    }
    #[test] 
    fn compress_and_decompress_with_gzip() {
        compress_and_decomress(CompressionType::GZIP)
   }
    #[test]
    fn compress_and_decompress_with_zstd() {
        compress_and_decomress(CompressionType::ZSTD)
    }
    #[test]
    fn compress_and_decompress_with_xz() {
        compress_and_decomress(CompressionType::XZ)
    }
    #[test]
    fn compress_and_decompress_with_lz4() {
        compress_and_decomress(CompressionType::LZ4)
    }
}
