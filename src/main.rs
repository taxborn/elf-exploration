use std::fs::File;
use std::process::{Command, ExitStatus};
use std::io::prelude::*;

// 0x7f + "ELF"
const MAGIC_BYTES: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

enum Endianness {
    Little = 1,
    Big = 2,
}

struct ElfHeader {
    pub e_ident_magic: [u8; 4], // Magic bytes, see MAGIC_BYTES const
    pub e_ident_class: u8,
    pub e_ident_data: u8,    // 1 = little endian, 2 = big endian
    pub e_ident_version: u8, // always set to 1 for the original and current version of ELF
    pub e_ident_osabi: u8,   // the target operating system ABI
    pub e_ident_abiversion: u8,
    pub e_ident_pad: [u8; 7], // currently unused, should be filled with 0's
    pub e_type: [u8; 2],
    pub e_machine: [u8; 2],
    pub e_version: [u8; 4], // set to 1
    pub e_entry: [u8; 8],   // memory address of the entry point
    pub e_phoff: [u8; 8],
    pub e_shoff: [u8; 8],
    pub e_flags: [u8; 4],     // ???
    pub e_ehsize: [u8; 2],    // size of this header. 64 bytes for 64-bit, 52 bytes for 32-bit
    pub e_phentsize: [u8; 2], // size of this header. 64 bytes for 64-bit, 52 bytes for 32-bit
    pub e_phnum: [u8; 2],     // Number of entries in the program header table
    pub e_shentsize: [u8; 2], // Size of a program header table entry
    pub e_shnum: [u8; 2],     // Number of entrties in the section header table
    pub e_shstrndx: [u8; 2],  // Index of the section header table entry that contains the section
                              // names
}

impl ElfHeader {
    fn new(ident_class: u8, endianness: Endianness) -> Self {
        assert!(ident_class == 1 || ident_class == 2);

        Self {
            e_ident_magic: MAGIC_BYTES,
            e_ident_class: ident_class,
            e_ident_data: endianness as u8,
            e_ident_version: 1,
            e_ident_osabi: 0x03,
            e_ident_abiversion: 0,
            e_ident_pad: [0; 7],
            e_type: [0x0, 0x02],
            e_machine: [0x0, 0x32],
            e_version: [0x0, 0x0, 0x0, 0x1],
            e_entry: [0; 8],
            e_phoff: [0; 8],
            e_shoff: [0; 8],
            e_flags: [0; 4],
            e_ehsize: [0; 2],
            e_phentsize: [0; 2],
            e_phnum: [0; 2],
            e_shentsize: [0; 2],
            e_shnum: [0; 2],
            e_shstrndx: [0; 2],
        }
    }

    fn write_to_file(&self, name: &str) -> std::io::Result<()> {
        let mut file = File::create(name)?;

        file.write(&self.e_ident_magic)?;
        file.write(&[self.e_ident_class])?;
        file.write(&[self.e_ident_data])?;
        file.write(&[self.e_ident_version])?;
        file.write(&[self.e_ident_osabi])?;
        file.write(&[self.e_ident_abiversion])?;
        file.write(&self.e_ident_pad)?;
        file.write(&self.e_type)?;
        file.write(&self.e_machine)?;
        file.write(&self.e_version)?;

        // If 32 bit, these only take up 4
        if self.e_ident_class == 2 {
            file.write(&[0x1; 8])?;
            file.write(&self.e_phoff)?;
            file.write(&self.e_shoff)?;
        } else {
            file.write(&[0x2; 4])?;
            file.write(&self.e_phoff[..4])?;
            file.write(&self.e_shoff[..4])?;
        }

        file.write(&self.e_flags)?;

        if self.e_ident_class == 2 {
            file.write(&[0x0, 64])?;
        } else {
            file.write(&[0x0, 52])?;
        }

        file.write(&self.e_phentsize)?;
        file.write(&self.e_phnum)?;
        file.write(&self.e_shentsize)?;
        file.write(&self.e_shnum)?;
        file.write(&self.e_shstrndx)?;

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    let file = ElfHeader::new(2, Endianness::Big);
    let name = "out-64bit.elf";
    file.write_to_file(name)?;

    println!("Checking {name}...");
    let output = Command::new("readelf")
        .arg("-h")
        .arg(name)
        .status()?;

    if !output.success() {
        println!("There was an error with {name}"); 
    }

    let file = ElfHeader::new(1, Endianness::Big);
    let name = "out-32bit.elf";
    file.write_to_file(name)?;

    println!("\nChecking {name}...");
    let output2 = Command::new("readelf")
        .arg("-h")
        .arg(name)
        .status()?;

    if !output2.success() {
        println!("There was an error with {name}"); 
    }

    Ok(())
}
