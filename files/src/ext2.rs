//! Minimal ext2 filesystem support for Polished OS
//!
//! This module provides basic read-only ext2 support for loading userland programs or config files.
//! It is designed for no_std environments and does not require alloc.

use core::ptr;

/// Represents an ext2 superblock (partial, only fields we need).
#[repr(C, packed)]
pub struct Ext2SuperBlock {
    pub inodes_count: u32,
    pub blocks_count: u32,
    pub reserved_blocks_count: u32,
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,
    pub first_data_block: u32,
    pub log_block_size: u32,
    // ... add more fields as needed ...
}

/// Represents an ext2 inode (partial, only fields we need).
#[repr(C, packed)]
pub struct Ext2Inode {
    pub mode: u16,
    pub uid: u16,
    pub size: u32,
    pub atime: u32,
    pub ctime: u32,
    pub mtime: u32,
    pub dtime: u32,
    pub gid: u16,
    pub links_count: u16,
    pub blocks: u32,
    pub flags: u32,
    pub osd1: u32,
    pub block: [u32; 15], // Pointers to blocks
                          // ... more fields, but not needed for minimal read ...
}

/// Represents an ext2 directory entry (partial, only fields we need).
#[repr(C, packed)]
pub struct Ext2DirEntry {
    pub inode: u32,
    pub rec_len: u16,
    pub name_len: u8,
    pub file_type: u8,
    // name follows (not null-terminated)
}

/// Minimal ext2 error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ext2Error {
    InvalidSuperblock,
    NotFound,
    IoError,
    Unsupported,
}

/// Minimal ext2 block device trait.
pub trait BlockDevice {
    fn read_block(&self, lba: u64, buf: &mut [u8]) -> Result<(), Ext2Error>;
}

/// Minimal ext2 filesystem struct.
pub struct Ext2<'a, D: BlockDevice> {
    pub device: &'a D,
    pub superblock: Ext2SuperBlock,
}

impl<'a, D: BlockDevice> Ext2<'a, D> {
    /// Create a new ext2 instance from a block device.
    pub fn new(device: &'a D) -> Result<Self, Ext2Error> {
        let mut buf = [0u8; 1024];
        device.read_block(2, &mut buf[0..512])?;
        device.read_block(3, &mut buf[512..1024])?;
        // Log the first 64 bytes of the buffer for debugging
        use core::fmt::Write;
        struct HexBuf<'a>(&'a mut [u8], usize);
        impl<'a> Write for HexBuf<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let end = self.1 + bytes.len();
                if end > self.0.len() {
                    return Err(core::fmt::Error);
                }
                self.0[self.1..end].copy_from_slice(bytes);
                self.1 = end;
                Ok(())
            }
        }
        for i in (0..64).step_by(16) {
            let chunk = &buf[i..i + 16];
            let mut hex = [0u8; 48];
            let hex_len = {
                let mut hexbuf = HexBuf(&mut hex, 0);
                for b in chunk.iter() {
                    let _ = write!(&mut hexbuf, "{:02x} ", b);
                }
                hexbuf.1
            };
            let hexstr = core::str::from_utf8(&hex[..hex_len]).unwrap_or("<hex error>");
            let mut msg = [0u8; 80];
            let msg_len = {
                let mut msgbuf = HexBuf(&mut msg, 0);
                let _ = write!(
                    &mut msgbuf,
                    "ext2 superblock[{:03}-{:03}]: {}",
                    i,
                    i + 15,
                    hexstr
                );
                msgbuf.1
            };
            let msgstr = core::str::from_utf8(&msg[..msg_len]).unwrap_or("<hex error>");
            polished_serial_logging::info(msgstr);
        }
        // Log the magic number bytes
        let mut s = [0u8; 8];
        let s_len = {
            let mut w = HexBuf(&mut s, 0);
            let _ = write!(&mut w, "{:02x} {:02x}", buf[56], buf[57]);
            w.1
        };
        let magic_str = core::str::from_utf8(&s[..s_len]).unwrap_or("<hex error>");
        polished_serial_logging::info("ext2 superblock magic bytes: ");
        polished_serial_logging::info(magic_str);
        let superblock = unsafe { ptr::read(buf.as_ptr() as *const Ext2SuperBlock) };
        if buf[56] != 0x53 || buf[57] != 0xEF {
            polished_serial_logging::warn("ext2: Invalid superblock magic!");
            return Err(Ext2Error::InvalidSuperblock);
        }
        polished_serial_logging::info("ext2: Superblock magic valid");
        // Debug: Dump the first 128 bytes of the buffer
        for i in (0..128).step_by(16) {
            let chunk = &buf[i..i + 16];
            let mut hex = [0u8; 48];
            let hex_len = {
                let mut hexbuf = HexBuf(&mut hex, 0);
                for b in chunk.iter() {
                    let _ = write!(&mut hexbuf, "{:02x} ", b);
                }
                hexbuf.1
            };
            let hexstr = core::str::from_utf8(&hex[..hex_len]).unwrap_or("<hex error>");
            let mut msg = [0u8; 80];
            let msg_len = {
                let mut msgbuf = HexBuf(&mut msg, 0);
                let _ = write!(
                    &mut msgbuf,
                    "ext2 superblock raw[{:03}-{:03}]: {}",
                    i,
                    i + 15,
                    hexstr
                );
                msgbuf.1
            };
            let msgstr = core::str::from_utf8(&msg[..msg_len]).unwrap_or("<hex error>");
            polished_serial_logging::info(msgstr);
        }
        Ok(Self { device, superblock })
    }
    /// EXTREMELY MINIMAL: Read the first data block of inode 12 (should be /myfile.txt for a fresh ext2)
    pub fn read_first_file_block(&self, buf: &mut [u8]) -> Result<(), Ext2Error> {
        // ext2 inode table usually starts at block 5 (for 1k block size), inode 12 is first file
        // This is a hack: real code should parse the directory and inode tables
        let block = 7; // This is a guess for demo, may need to adjust
        self.device.read_block(block, buf)
    }
    /// Read an inode by number (1-based, as in ext2 spec)
    pub fn read_inode(&self, inode_num: u32, buf: &mut Ext2Inode) -> Result<(), Ext2Error> {
        // Calculate inode table location
        let block_size = 1024 << self.superblock.log_block_size;
        let inodes_per_block = block_size / core::mem::size_of::<Ext2Inode>();
        let inode_index = inode_num - 1;
        let inode_table_block = 5; // For 1k block size, inode table starts at block 5 (hardcoded for now)
        let block = inode_table_block + (inode_index as usize / inodes_per_block);
        let offset = (inode_index as usize % inodes_per_block) * core::mem::size_of::<Ext2Inode>();
        let mut block_buf = [0u8; 1024];
        self.device.read_block(block as u64, &mut block_buf)?;
        let src = &block_buf[offset..offset + core::mem::size_of::<Ext2Inode>()];
        unsafe {
            ptr::copy_nonoverlapping(
                src.as_ptr(),
                buf as *mut Ext2Inode as *mut u8,
                core::mem::size_of::<Ext2Inode>(),
            );
        }
        Ok(())
    }
    /// Read directory entries from a directory inode, searching for a file by name.
    pub fn find_file_in_dir(
        &self,
        dir_inode: &Ext2Inode,
        filename: &str,
        dir_block_buf: &mut [u8],
    ) -> Result<u32, Ext2Error> {
        let block_size = 1024 << self.superblock.log_block_size;
        // Copy block array to avoid unaligned reference
        let blocks = dir_inode.block;
        for &block in blocks.iter().take(12) {
            if block == 0 {
                continue;
            }
            self.device
                .read_block(block as u64, &mut dir_block_buf[..block_size])?;
            let mut offset = 0;
            while offset < block_size {
                let entry = unsafe { &*(dir_block_buf[offset..].as_ptr() as *const Ext2DirEntry) };
                if entry.inode == 0 {
                    break;
                }
                let name_slice = &dir_block_buf[offset + 8..offset + 8 + (entry.name_len as usize)];
                if let Ok(name) = core::str::from_utf8(name_slice) {
                    if name == filename {
                        return Ok(entry.inode);
                    }
                }
                if entry.rec_len == 0 {
                    break;
                }
                offset += entry.rec_len as usize;
            }
        }
        Err(Ext2Error::NotFound)
    }

    /// Read the first data block of a file by name in the root directory.
    pub fn read_file_first_block(&self, filename: &str, buf: &mut [u8]) -> Result<(), Ext2Error> {
        // 1. Read root inode (inode 2)
        let mut root_inode = Ext2Inode {
            mode: 0,
            uid: 0,
            size: 0,
            atime: 0,
            ctime: 0,
            mtime: 0,
            dtime: 0,
            gid: 0,
            links_count: 0,
            blocks: 0,
            flags: 0,
            osd1: 0,
            block: [0; 15],
        };
        self.read_inode(2, &mut root_inode)?;
        // 2. Find file in root directory
        let block_size = 1024 << self.superblock.log_block_size;
        let mut dir_block_buf = [0u8; 4096]; // Support up to 4k block size
        let file_inode_num = self.find_file_in_dir(&root_inode, filename, &mut dir_block_buf)?;
        // 3. Read file inode
        let mut file_inode = Ext2Inode {
            mode: 0,
            uid: 0,
            size: 0,
            atime: 0,
            ctime: 0,
            mtime: 0,
            dtime: 0,
            gid: 0,
            links_count: 0,
            blocks: 0,
            flags: 0,
            osd1: 0,
            block: [0; 15],
        };
        self.read_inode(file_inode_num, &mut file_inode)?;
        // 4. Read first data block of file
        let first_block = file_inode.block[0];
        if first_block == 0 {
            return Err(Ext2Error::NotFound);
        }
        self.device.read_block(first_block as u64, buf)?;
        Ok(())
    }
    // Add more methods: read_dir, read_file, etc.
}
