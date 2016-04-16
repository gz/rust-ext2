#![no_std]
#![feature(const_fn)]
#![feature(collections)]

#![crate_name = "ext2"]
#![crate_type = "lib"]

extern crate collections;

use core::mem::{size_of, transmute, zeroed};
//use core::str;
//use core::slice;
//use core::default;
use core::fmt;
use collections::{String};

use std::fs::File;
use std::io::prelude::*;


#[macro_use]
extern crate std;

pub trait Disk {
    fn name(&self) -> String;
    fn read(&mut self, block: u64, buffer: &mut [u8]) -> Result<(), usize>;
    fn write(&mut self, block: u64, buffer: &[u8]) -> Result<(), usize>;
}

struct ExtFS<'a> {
    disk: &'a (Disk + 'a),
}

impl<'a> ExtFS<'a> {
    const fn new(disk: &'a Disk) -> ExtFS {
        ExtFS { disk: disk }
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
        try!(write!(f, "\tBlock size: {}\n", self.log_block_size));
        try!(write!(f, "\tFragment size: {}\n", self.log_frag_size));
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
struct GroupDesc {
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


const EXT2_NDIR_BLOCKS: usize = 12;
const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
const EXT2_DIND_BLOCK: usize = (EXT2_IND_BLOCK + 1);
const EXT2_TIND_BLOCK: usize = (EXT2_DIND_BLOCK + 1);
const EXT2_N_BLOCKS: usize = (EXT2_TIND_BLOCK + 1);

pub struct INode {
     /// File mode
     mode: u16,
     /// Low 16 bits of Owner Uid
     uid: u16,
     /// Size in bytes
     size: u32,
     /// Access time
     atime: u32,
     /// Creation time
     ctime: u32,
     /// Modification time
     mtime: u32,
     /// Deletion Time
     dtime: u32,
     /// Low 16 bits of Group Id
     gid: u16,
     /// Links count
     links_count: u16,
     /// Blocks count
     blocks: u32,
     /// File flags
     flags: u32,
     l_reserved1: u32,
     /// Pointers to blocks
     block: [u32; EXT2_N_BLOCKS],
     /// File version (for NFS)
     generation: u32,
     /// File ACL
     file_acl: u32,
     /// Directory ACL
     dir_acl: u32,
     /// Fragment address
     faddr: u32,
     /// Fragment number
     l_frag: u8,
     /// Fragment size
     l_fsize: u8,
     pad1: u16,
     l_uid_high: u16,
     l_gid_high: u16,
     l_reserved2: u32,
}

impl core::default::Default for INode {
    fn default() -> INode {
        unsafe { zeroed() }
    }
}

impl core::fmt::Debug for INode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "INode\n"));
        try!(write!(f, "\tFile mode: {}\n", self.mode));
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
        //try!(write!(f, "\tPointers to blocks: {}\n", self.block));
        try!(write!(f, "\tFile version (for NFS): {}\n", self.generation));
        try!(write!(f, "\tFile ACL: {}\n", self.file_acl));
        try!(write!(f, "\tDirectory ACL: {}\n", self.dir_acl));
        try!(write!(f, "\tFragment address: {}\n", self.faddr));
        try!(write!(f, "\tFragment number: {}\n", self.l_frag));
        write!(f, "\tFragment size: {}\n", self.l_fsize)
    }
}

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


/*impl core::default::Default for DirEntry {
    fn default() -> DirEntry {
        unsafe { zeroed() }
    }
}*/

impl core::fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "DirEntry\n"));
        try!(write!(f, "\tInode number: {}\n", self.inode));
        try!(write!(f, "\tDirectory entry length: {}\n", self.rec_len));
        write!(f, "\tName length: {}\n", self.name_len)
        //try!(write!(f, "\tFile name, up to EXT2_NAME_LEN: {}\n", self.name));
    }
}
