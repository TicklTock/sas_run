use std::path::{Path, PathBuf};
use std::{os::windows::process::CommandExt, process::Command};
use winreg::enums::*;
use winreg::RegKey;


pub enum Encoding {
    UTF8,
    EucCn,
}

pub struct Sas{
    cmd: Command,
    config_path: PathBuf,
    pub sas_path: PathBuf,
    pub log_path: Option<PathBuf>,
}

impl Sas {
    pub fn new<T: AsRef<Path>>(encoding: Encoding, sas_path: T, log_path: Option<T>) -> Self{
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
    pub fn run(&mut self) -> anyhow::Result<()> {
        let config_path = self.config_path.to_str().unwrap();
        let sas_path = self.sas_path.to_str().unwrap();

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
    fn test() -> anyhow::Result<()> {
        let mut sas_dm = Sas::new(Encoding::UTF8, r"sample.sas", Some("."));
        sas_dm.run()?;
        Ok(())
    }
}