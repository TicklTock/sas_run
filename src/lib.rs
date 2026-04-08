use std::path::{Path, PathBuf};
use std::{os::windows::process::CommandExt, process::Command, fmt::Debug};
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use winreg::enums::*;
use winreg::RegKey;
use tokio::process::Command as TokioCommand;



#[derive(Error)]
pub enum SasError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl Debug for SasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Encoding {
    UTF8,
    EucCn,
}

#[derive(Debug)]
pub struct Sas{
    cmd: Command,
    config_path: PathBuf,
    pub sas_path: PathBuf,
    pub log_path: Option<PathBuf>,
}

impl Sas {
    pub fn new<T: AsRef<Path>, U: AsRef<Path>>(encoding: Encoding, sas_path: T, log_path: Option<U>) -> Self{
        let config_path = {
            match encoding {
                Encoding::UTF8 => Path::join(Path::new(&Sas::get_sas_path()), r"nls\u8\sasv9.cfg"),
                Encoding::EucCn => Path::join(Path::new(&Sas::get_sas_path()), r"nls\zh\sasv9.cfg"),
            }
        };
        let log_path = {
            match log_path {
                Some(path) => Some(path.as_ref().to_path_buf()),
                None => {
                    None
                }
                
            }
        };
        Self { cmd: Command::new("cmd.exe"), config_path: config_path, sas_path: sas_path.as_ref().to_path_buf(), log_path }
    }
    pub fn run(&mut self) -> Result<(), SasError> {
        let config_path = self.config_path.to_str().unwrap();
        let sas_path = self.sas_path.strip_prefix(r"\\?\").unwrap_or(&self.sas_path).to_str().unwrap();

        if !std::fs::exists(config_path)? {
            return Err(SasError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("SAS config file not found: {}", config_path))));
        }

        if !std::fs::exists(sas_path)? {
            return Err(SasError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("SAS file not found: {}", sas_path))));
        }

        let log_args = {
            if self.log_path.is_none(){
                format!("-NOLOG")
            }else{
                let path = self.log_path.clone().unwrap();
                if !std::fs::exists(&path)? {
                    std::fs::create_dir(&path)?;
                }
                format!("-LOG {}", path.to_str().unwrap())
            }
        };
        
        self.cmd
            .raw_arg("/c")
            .raw_arg("start")
            .raw_arg("/w")
            .arg("sas batch")
            .raw_arg("sas")
            .raw_arg("-CONFIG")
            .arg(config_path)
            .raw_arg("-SYSIN")
            .arg(sas_path)
            .raw_arg(log_args)
            .raw_arg("-nosplash")
            .raw_arg("-nologo")
            .raw_arg("-icon");
        self.cmd.output()?;
        Ok(())
    }
    // 异步run
    pub async fn run_async(&mut self) -> Result<(), SasError> {
        

        let mut cmd = TokioCommand::new("cmd.exe");
        let config_path = self.config_path.to_str().unwrap();
        let sas_path = self.sas_path.strip_prefix(r"\\?\").unwrap_or(&self.sas_path).to_str().unwrap();

        if !std::fs::exists(config_path)? {
            return Err(SasError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("SAS config file not found: {}", config_path))));
        }

        if !std::fs::exists(sas_path)? {
            return Err(SasError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("SAS file not found: {}", sas_path))));
        }

        let log_args = {
            if self.log_path.is_none(){
                format!("-NOLOG")
            }else{
                let path = self.log_path.clone().unwrap();
                if !std::fs::exists(&path)? {
                    std::fs::create_dir(&path)?;
                }
                format!("-LOG {}", path.to_str().unwrap())
            }
        };
        
        cmd
            .raw_arg("/c")
            .raw_arg("start")
            .raw_arg("/w")
            .arg("sas batch")
            .raw_arg("sas")
            .raw_arg("-CONFIG")
            .arg(config_path)
            .raw_arg("-SYSIN")
            .arg(sas_path)
            .raw_arg(log_args)
            .raw_arg("-nosplash")
            .raw_arg("-nologo")
            .raw_arg("-icon");
        cmd.output().await?;
        Ok(())
    }

    pub fn get_sas_path() -> String {
        let hklm = RegKey::predef(HKEY_CLASSES_ROOT);
        let cur_sas = hklm.open_subkey("SAS.Application\\shell\\Open\\Command").unwrap();
        let sas_path: String = cur_sas.get_value("").unwrap();
        let sas_cfg = sas_path.rsplit('"').collect::<Vec<_>>()[1].to_string();
        sas_cfg.split('\\').filter(|x| !x.to_lowercase().contains(".cfg")).collect::<Vec<_>>().join("\\")
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_u8() -> Result<(), SasError> {
        let path = PathBuf::from(r".\test\sample.sas");
        let mut sas_dm = Sas::new(Encoding::UTF8, path, Some("test"));
        sas_dm.run()?;
        Ok(())
    }
    
    #[test]
    fn test_zh() -> Result<(), SasError> {

        let mut sas_dm = Sas::new(Encoding::EucCn, r".\test\sample.sas", Some("test"));
        sas_dm.run()?;
        Ok(())
    }

    #[test]
    fn test_u8_async() -> Result<(), SasError> {
        let mut sas_dm = Sas::new(Encoding::UTF8, r".\test\sample.sas", Some("test"));
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            sas_dm.run_async().await
        })?;        
        Ok(())
    }

    #[test]
    fn test_zh_async() -> Result<(), SasError> {
        let mut sas_dm = Sas::new(Encoding::EucCn, r".\test\sample.sas", Some("test"));
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            sas_dm.run_async().await
        })?;        
        Ok(())
    }
}