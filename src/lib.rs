#![no_std]
#![feature(const_fn)]
#![feature(collections)]
#![feature(alloc)]

#![crate_name = "ext2"]
#![crate_type = "lib"]

extern crate collections;
extern crate alloc;

use core::mem::{size_of, transmute, zeroed};
use core::mem;
//use core::str;
//use core::slice;
//use core::default;
use core::fmt;
use core::slice;
use collections::{String, Vec};
use collections::boxed::{Box};
use alloc::raw_vec::{RawVec};

use std::fs::File;
use std::io::prelude::*;

// TODO: we would like to be libcore only:
#[macro_use]
extern crate std;
//use std::collections::HashMap;
//use std::boxed::Box;


pub trait Disk {
    fn name(&self) -> String;
    fn read(&mut self, block: u64, buffer: &mut [u8]) -> Result<(), usize>;
    fn write(&mut self, block: u64, buffer: &[u8]) -> Result<(), usize>;
}

/*
let len = self.len();
let buf = RawVec::with_capacity(len);
unsafe {
    ptr::copy_nonoverlapping(self.as_ptr(), buf.ptr(), len);
    mem::transmute(buf.into_box()) // bytes to str ~magic
}*/

#[derive(Debug)]
pub enum BlockStorageError { BAD }

pub enum BlockDataType {
    Raw,
    GroupDesc,
    INodeTable,
    BlockBitmap,
}

pub struct Block {
    block_number: u64,
    dirty: bool,
    buffer: RawVec<u8>,
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        write!(f, "Block {{ nr: {} dirty: {} }}", self.block_number, self.dirty)
    }
}

impl Block {

    fn new(block: u64, buffer: RawVec<u8>) -> Block {
        Block { dirty: false, block_number: block, buffer: buffer }
    }

    pub fn load_from_disk(disk: &mut Box<Disk>, block: u64, length: usize) -> Result<Block, BlockStorageError> {
        let buffer = RawVec::with_capacity(length);
        let slice: &mut [u8] = unsafe { slice::from_raw_parts_mut(buffer.ptr(), length) };

        disk.read(block as u64, slice).unwrap();
        Ok(Block::new(block, buffer))
    }

    pub unsafe fn as_group_descriptor<'a>(&'a self) -> &'a GroupDesc {
         mem::transmute::<*const u8, &GroupDesc>(self.buffer.ptr())
    }

    pub unsafe fn as_inode_slice<'a>(&'a self, how_many: usize) -> &'a [INode] {
        let ptr = mem::transmute::<*const u8, &INode>(self.buffer.ptr());
        slice::from_raw_parts(ptr, how_many)
    }

    pub unsafe fn directory_iter<'a>(&'a self, how_many: usize) -> DirIter<'a> {
        DirIter { block: self.buffer(), offset: 0 }
    }

    pub fn buffer<'a>(&'a self) -> &'a [u8] {
        unsafe { slice::from_raw_parts(self.buffer.ptr(), self.buffer.cap()) }
    }

    pub fn buffer_mut<'a>(&'a mut self) -> &'a mut [u8] {
        self.dirty = true;
        unsafe { slice::from_raw_parts_mut(self.buffer.ptr(), self.buffer.cap()) }
    }
}

pub struct DirIter<'a> {
    block: &'a [u8],
    offset: usize
}

//impl<'a, T> Iterator<&'a T> for Items<'a, T>.

impl<'a> Iterator for DirIter<'a> {
    type Item = &'a DirEntry;

    #[inline]
    fn next(&mut self) -> Option<&'a DirEntry> {

        if self.offset < self.block.len() {
            let name_len = unsafe { mem::transmute::<*const u8, *const u16>(&self.block[self.offset+6]) };
            let slice: &[u8] = unsafe { slice::from_raw_parts(&self.block[self.offset], *name_len as usize) };
            let dir_entry = unsafe { mem::transmute::<&'a [u8], &'a DirEntry>(slice) };

            //println!("{:?}", dir_entry);
            self.offset += dir_entry.rec_len as usize;
            return Some(dir_entry);
        }

        return None;
    }
}

pub struct BlockStorageService {
    disk: Box<Disk>,
    block_size: usize,
}

impl BlockStorageService {

    pub fn new(disk: Box<Disk>, block_size: usize) -> BlockStorageService {
        BlockStorageService { disk: disk, block_size: block_size }
    }

    pub fn get(&mut self, block: u64) -> Result<Block, BlockStorageError> {
        Block::load_from_disk(&mut self.disk, block, self.block_size)
    }

    fn write(&mut self, b: Block) -> Result<(), BlockStorageError> {
        Ok(())
    }

}

pub struct ExtFS {
    pub bs: BlockStorageService,
    pub sb: SuperBlock,
}

impl ExtFS {

    pub fn new(mut disk: Box<Disk>) -> ExtFS {
        let sb: SuperBlock = SuperBlock::from_disk(&mut disk);
        let bs: BlockStorageService = BlockStorageService::new(disk, sb.block_size() as usize);
        ExtFS { bs: bs, sb: sb }
    }

    pub fn group_descriptor_table(&mut self) -> Result<Block, BlockStorageError> {
        let block_number: u64 = self.sb.first_data_block as u64 + 1;
        self.bs.get(block_number)
    }

    pub fn inode_table(&mut self, gd: &GroupDesc) -> Result<Block, BlockStorageError> {
        self.bs.get(gd.inode_table as u64)
    }
}

#[repr(C)]
pub struct SuperBlock {
    /// Inodes count
    pub inodes_count: u32,
    /// Blocks count
    pub blocks_count: u32,
    /// Reserved blocks count
    pub r_blocks_count: u32,
    /// Free blocks count
    pub free_blocks_count: u32,
    /// Free inodes count
    pub free_inodes_count: u32,
    /// First Data Block
    pub first_data_block: u32,
    /// Block size
    pub log_block_size: u32,
    /// Fragment size
    pub log_frag_size: u32,
    /// # Blocks per group
    pub blocks_per_group: u32,
    /// # Fragments per group
    pub frags_per_group: u32,
    /// # Inodes per group
    pub inodes_per_group: u32,
    /// Mount time
    pub mtime: u32,
    /// Write time
    pub wtime: u32,
    /// Mount count
    pub mnt_count: u16,
    /// Maximal mount count
    pub max_mnt_count: u16,
    /// Magic signature
    pub magic: u16,
    /// File system state
    pub state: u16,
    /// Behaviour when detecting errors
    pub errors: u16,
    /// minor revision level
    pub minor_rev_level: u16,
    /// time of last check
    pub lastcheck: u32,
    /// max. time between checks
    pub checkinterval: u32,
    /// OS
    pub creator_os: u32,
    /// Revision level
    pub rev_level: u32,
    /// Default uid for reserved blocks
    pub def_resuid: u16,
    /// Default gid for reserved blocks
    pub def_resgid: u16,

	/*
	 * These fields are for EXT2_DYNAMIC_REV superblocks only.
	 *
	 * Note: the difference between the compatible feature set and
	 * the incompatible feature set is that if there is a bit set
	 * in the incompatible feature set that the kernel doesn't
	 * know about, it should refuse to mount the filesystem.
	 *
	 * e2fsck's requirements are more strict; if it doesn't know
	 * about a feature in either the compatible or incompatible
	 * feature set, it must abort and not try to meddle with
	 * things it doesn't understand...
	 */
     pub first_ino: u32, 		/* First non-reserved inode */
     pub inode_size: u16, 		/* size of inode structure */
     pub block_group_nr: u16, 	/* block group # of this superblock */
     pub feature_compat: u32, 	/* compatible feature set */
     pub feature_incompat: u32, 	/* incompatible feature set */
     pub feature_ro_compat: u32, 	/* readonly-compatible feature set */
     pub uuid: [u8; 16],		/* 128-bit uuid for volume */
     pub volume_name: [u8; 16], 	/* volume name */
     pub last_mounted: [u8; 64], 	/* directory where last mounted */
     pub algorithm_usage_bitmap: u32, /* For compression */

	/*
	 * Performance hints.  Directory preallocation should only
	 * happen if the EXT2_COMPAT_PREALLOC flag is on.
	 */
     pub prealloc_blocks: u8,	/* Nr of blocks to try to preallocate*/
     pub prealloc_dir_blocks: u8,	/* Nr to preallocate for dirs */
     pub padding1: u16,

	/*
	 * Journaling support valid if EXT3_FEATURE_COMPAT_HAS_JOURNAL set.
	 */
     pub journal_uuid: [u8; 16],	/* uuid of journal superblock */
     pub journal_inum: u32,		/* inode number of journal file */
     pub journal_dev: u32,		/* device number of journal file */
     pub last_orphan: u32,		/* start of list of inodes to delete */
     pub hash_seed: [u32; 4],		/* HTREE hash seed */
     pub def_hash_version: u8,	/* Default hash version to use */
     pub reserved_char_pad: u8,
     pub reserved_word_pad: u16,
     pub default_mount_opts: u32,
     pub first_meta_bg: u32, 	/* First metablock block group */
     pub reserved: [u32; 190],	/* Padding to the end of the block */
}

impl SuperBlock {

    fn from_disk<'a>(disk: &mut Box<Disk>) -> SuperBlock {
        let mut sb = SuperBlock::default();
        let mut buffer: &mut [u8; 1024] = unsafe { mem::transmute::<&mut SuperBlock, &mut [u8; 1024]>(&mut sb) };
        disk.read(1, buffer).unwrap();

        assert!(sb.magic == 0xEF53);
        sb
    }

    fn block_size(&self) -> u64 {
        1024 << self.log_block_size
    }

    fn frag_size(&self) -> u64 {
        if self.log_frag_size > 0 {
            1024 << self.log_frag_size
        }
        else {
            1024 >> -(self.log_frag_size as i32)
        }

    }
}

impl core::default::Default for SuperBlock {
    fn default() -> SuperBlock {
        unsafe { zeroed() }
    }
}

impl core::fmt::Debug for SuperBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "SuperBlock\n"));
        try!(write!(f, "\tInodes count: {}\n", self.inodes_count));
        try!(write!(f, "\tBlocks count: {}\n", self.blocks_count));
        try!(write!(f, "\tReserved blocks count: {}\n", self.r_blocks_count));
        try!(write!(f, "\tFree blocks count: {}\n", self.free_blocks_count));
        try!(write!(f, "\tFree inodes count: {}\n", self.free_inodes_count));
        try!(write!(f, "\tFirst Data Block: {}\n", self.first_data_block));
        try!(write!(f, "\tBlock size: {}\n", self.block_size()));
        try!(write!(f, "\tFragment size: {}\n", self.frag_size()));
        try!(write!(f, "\t# Blocks per group: {}\n", self.blocks_per_group));
        try!(write!(f, "\t# Fragments per group: {}\n", self.frags_per_group));
        try!(write!(f, "\t# Inodes per group: {}\n", self.inodes_per_group));
        try!(write!(f, "\tMount time: {}\n", self.mtime));
        try!(write!(f, "\tWrite time: {}\n", self.wtime));
        try!(write!(f, "\tMount count: {}\n", self.mnt_count));
        try!(write!(f, "\tMaximal mount count: {}\n", self.max_mnt_count));
        try!(write!(f, "\tMagic signature: {}\n", self.magic));
        try!(write!(f, "\tFile system state: {}\n", self.state));
        try!(write!(f, "\tBehaviour when detecting errors: {}\n", self.errors));
        try!(write!(f, "\tminor revision level: {}\n", self.minor_rev_level));
        try!(write!(f, "\ttime of last check: {}\n", self.lastcheck));
        try!(write!(f, "\tmax. time between checks: {}\n", self.checkinterval));
        try!(write!(f, "\tOS: {}\n", self.creator_os));
        try!(write!(f, "\tRevision level: {}\n", self.rev_level));
        try!(write!(f, "\tDefault uid for reserved blocks: {}\n", self.def_resuid));
        write!(f, "\tDefault gid for reserved blocks: {}\n", self.def_resgid)

    }
}

#[repr(C)]
pub struct GroupDesc {
     /// Blocks bitmap block
     block_bitmap: u32,
     /// Inodes bitmap block
     inode_bitmap: u32,
     /// Inodes table block
     inode_table: u32,
     /// Free blocks count
     free_blocks_count: u16,
     /// Free inodes count
     free_inodes_count: u16,
     /// Directories count
     used_dirs_count: u16,
     pad: u16,
     reserved: [u32; 3]
}

impl GroupDesc {

    fn from_fs(fs: &mut ExtFS, block: u32) -> GroupDesc {
        let mut gd = GroupDesc::default();
        let mut buffer: &mut [u8; 32] = unsafe { mem::transmute::<&mut GroupDesc, &mut [u8; 32]>(&mut gd) };

        let b: Block = fs.bs.get(block as u64).unwrap();
        gd
    }

}

impl core::default::Default for GroupDesc {
    fn default() -> GroupDesc {
        unsafe { zeroed() }
    }
}

impl core::fmt::Debug for GroupDesc {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "GroupDesc\n"));
        try!(write!(f, "\tBlocks bitmap block: {}\n", self.block_bitmap));
        try!(write!(f, "\tInodes bitmap block: {}\n", self.inode_bitmap));
        try!(write!(f, "\tInodes table block: {}\n", self.inode_table));
        try!(write!(f, "\tFree blocks count: {}\n", self.free_blocks_count));
        try!(write!(f, "\tFree inodes count: {}\n", self.free_inodes_count));
        write!(f, "\tDirectories count: {}\n", self.used_dirs_count)
    }
}


// file format
/// socket
pub const EXT2_S_IFSOCK: u16 = 0xC000;
/// symbolic link
pub const EXT2_S_IFLNK: u16 = 0xA000;
/// regular file
pub const EXT2_S_IFREG: u16 = 0x8000;
/// block device
pub const EXT2_S_IFBLK: u16 = 0x6000;
/// directory
pub const EXT2_S_IFDIR: u16 = 0x4000;
/// character device
pub const EXT2_S_IFCHR: u16 = 0x2000;
/// fifo
pub const EXT2_S_IFIFO: u16 = 0x1000;


// process execution user/group override
/// Set process User ID
pub const EXT2_S_ISUID: u16 = 0x0800;
/// Set process Group ID
pub const EXT2_S_ISGID: u16 = 0x0400;
/// sticky bit
pub const EXT2_S_ISVTX: u16 = 0x0200;


// access rights
/// user read
pub const EXT2_S_IRUSR: u16 = 0x0100;
/// user write
pub const EXT2_S_IWUSR: u16 = 0x0080;
/// user execute
pub const EXT2_S_IXUSR: u16 = 0x0040;
/// group read
pub const EXT2_S_IRGRP: u16 = 0x0020;
/// group write
pub const EXT2_S_IWGRP: u16 = 0x0010;
/// group execute
pub const EXT2_S_IXGRP: u16 = 0x0008;
/// others read
pub const EXT2_S_IROTH: u16 = 0x0004;
/// others write
pub const EXT2_S_IWOTH: u16 = 0x0002;
/// others execute
pub const EXT2_S_IXOTH: u16 = 0x0001;


// Defined Reserved Inodes
/// bad blocks inode
pub const EXT2_BAD_INO: usize = 1;
/// root directory inode
pub const EXT2_ROOT_INO: usize = 2;
/// ACL index inode (deprecated?)
pub const EXT2_ACL_IDX_INO: usize = 3;
/// ACL data inode (deprecated?)
pub const EXT2_ACL_DATA_INO: usize = 4;
/// boot loader inode
pub const EXT2_BOOT_LOADER_INO: usize = 5;
/// undelete directory inode
pub const EXT2_UNDEL_DIR_INO: usize = 6;


const EXT2_NDIR_BLOCKS: usize = 12;
const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
const EXT2_DIND_BLOCK: usize = (EXT2_IND_BLOCK + 1);
const EXT2_TIND_BLOCK: usize = (EXT2_DIND_BLOCK + 1);
const EXT2_N_BLOCKS: usize = (EXT2_TIND_BLOCK + 1);

#[repr(C)]
pub struct INode {
     /// File mode
     pub mode: u16,
     /// Low 16 bits of Owner Uid
     pub uid: u16,
     /// Size in bytes
     pub size: u32,
     /// Access time
     pub atime: u32,
     /// Creation time
     pub ctime: u32,
     /// Modification time
     pub mtime: u32,
     /// Deletion Time
     pub dtime: u32,
     /// Low 16 bits of Group Id
     pub gid: u16,
     /// Links count
     pub links_count: u16,
     /// Blocks count
     pub blocks: u32,
     /// File flags
     pub flags: u32,
     reserved1: u32,
     /// Pointers to blocks
     pub block: [u32; EXT2_N_BLOCKS],
     /// File version (for NFS)
     pub generation: u32,
     /// File ACL
     pub file_acl: u32,
     /// Directory ACL
     pub dir_acl: u32,
     /// Fragment address
     pub faddr: u32,
     /// Fragment number
     pub l_frag: u8,
     /// Fragment size
     pub l_fsize: u8,
     pad1: u16,
     l_uid_high: u16,
     l_gid_high: u16,
     l_reserved2: u32,
}

impl INode {

    pub fn size(&self) -> u64 {
        ((self.dir_acl as u64) << 32) | self.size as u64
    }

    pub fn file_format(&self) -> &'static str {
        let format = self.mode & 0xF000;

        if format == EXT2_S_IFSOCK {
            return "socket";
        }

        if format == EXT2_S_IFLNK {
            return "symbolic link";
        }

        if format == EXT2_S_IFREG {
            return "regular file";
        }

        if format == EXT2_S_IFBLK {
            return "block device";
        }

        if format == EXT2_S_IFDIR {
            return "directory";
        }

        if format == EXT2_S_IFCHR {
            return "character device";
        }

        if format == EXT2_S_IFIFO {
            return "fifo";
        }

        "unknown"
    }
}

impl core::default::Default for INode {
    fn default() -> INode {
        unsafe { zeroed() }
    }
}

impl core::fmt::Debug for INode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "INode\n"));
        try!(write!(f, "\tFile mode: {} ({})\n", self.mode, self.file_format()));
        try!(write!(f, "\tLow 16 bits of Owner Uid: {}\n", self.uid));
        try!(write!(f, "\tSize in bytes: {}\n", self.size));
        try!(write!(f, "\tAccess time: {}\n", self.atime));
        try!(write!(f, "\tCreation time: {}\n", self.ctime));
        try!(write!(f, "\tModification time: {}\n", self.mtime));
        try!(write!(f, "\tDeletion Time: {}\n", self.dtime));
        try!(write!(f, "\tLow 16 bits of Group Id: {}\n", self.gid));
        try!(write!(f, "\tLinks count: {}\n", self.links_count));
        try!(write!(f, "\tBlocks count: {}\n", self.blocks));
        try!(write!(f, "\tFile flags: {}\n", self.flags));
        try!(write!(f, "\tPointers to blocks: {} {} {} {} {} {}\n",
                       self.block[0], self.block[1], self.block[2], self.block[3], self.block[4], self.block[5]));
        try!(write!(f, "\tFile version (for NFS): {}\n", self.generation));
        try!(write!(f, "\tFile ACL: {}\n", self.file_acl));
        try!(write!(f, "\tDirectory ACL: {}\n", self.dir_acl));
        try!(write!(f, "\tFragment address: {}\n", self.faddr));
        try!(write!(f, "\tFragment number: {}\n", self.l_frag));
        write!(f, "\tFragment size: {}\n", self.l_fsize)
    }
}

// DirEntryIter

pub struct DirEntry {
    /// Inode number
    inode: u32,
    /// Directory entry length
    rec_len: u16,
    /// Name length
    name_len: u16,
    /// File name, up to EXT2_NAME_LEN
    name: [u8]
}

impl DirEntry {
    /*pub fn name<'a>(&'a self) -> &'a str {

    }*/
}

impl core::fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "DirEntry\n"));
        try!(write!(f, "\tInode number: {}\n", self.inode));
        try!(write!(f, "\tDirectory entry length: {}\n", self.rec_len));
        write!(f, "\tName length: {}\n", self.name_len)
        //try!(write!(f, "\tFile name, up to EXT2_NAME_LEN: {}\n", self.name));
    }
}
