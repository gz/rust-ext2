#![no_std]

#![crate_name = "ext2"]
#![crate_type = "lib"]


use core::mem::{size_of, transmute, zeroed};
use core::str;
use core::slice;
use core::default;
use core::fmt;

#[macro_use]
extern crate std;

#[repr(C, packed)]
pub struct Ext2SuperBlock {
    /// Inodes count
    pub s_inodes_count: u32,
    /// Blocks count
    pub s_blocks_count: u32,
    /// Reserved blocks count
    pub s_r_blocks_count: u32,
    /// Free blocks count
    pub s_free_blocks_count: u32,
    /// Free inodes count
    pub s_free_inodes_count: u32,
    /// First Data Block
    pub s_first_data_block: u32,
    /// Block size
    pub s_log_block_size: u32,
    /// Fragment size
    pub s_log_frag_size: u32,
    /// # Blocks per group
    pub s_blocks_per_group: u32,
    /// # Fragments per group
    pub s_frags_per_group: u32,
    /// # Inodes per group
    pub s_inodes_per_group: u32,
    /// Mount time
    pub s_mtime: u32,
    /// Write time
    pub s_wtime: u32,
    /// Mount count
    pub s_mnt_count: u16,
    /// Maximal mount count
    pub s_max_mnt_count: u16,
    /// Magic signature
    pub s_magic: u16,
    /// File system state
    pub s_state: u16,
    /// Behaviour when detecting errors
    pub s_errors: u16,
    /// minor revision level
    pub s_minor_rev_level: u16,
    /// time of last check
    pub s_lastcheck: u32,
    /// max. time between checks
    pub s_checkinterval: u32,
    /// OS
    pub s_creator_os: u32,
    /// Revision level
    pub s_rev_level: u32,
    /// Default uid for reserved blocks
    pub s_def_resuid: u16,
    /// Default gid for reserved blocks
    pub s_def_resgid: u16,

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
	s_first_ino: u32, 		/* First non-reserved inode */
	s_inode_size: u16, 		/* size of inode structure */
	s_block_group_nr: u16, 	/* block group # of this superblock */
	s_feature_compat: u32, 	/* compatible feature set */
	s_feature_incompat: u32, 	/* incompatible feature set */
	s_feature_ro_compat: u32, 	/* readonly-compatible feature set */
	s_uuid: [u8; 16],		/* 128-bit uuid for volume */
	s_volume_name: [u8; 16], 	/* volume name */
	s_last_mounted: [u8; 64], 	/* directory where last mounted */
	s_algorithm_usage_bitmap: u32, /* For compression */

	/*
	 * Performance hints.  Directory preallocation should only
	 * happen if the EXT2_COMPAT_PREALLOC flag is on.
	 */
	s_prealloc_blocks: u8,	/* Nr of blocks to try to preallocate*/
	s_prealloc_dir_blocks: u8,	/* Nr to preallocate for dirs */
	s_padding1: u16,

	/*
	 * Journaling support valid if EXT3_FEATURE_COMPAT_HAS_JOURNAL set.
	 */
	s_journal_uuid: [u8; 16],	/* uuid of journal superblock */
	s_journal_inum: u32,		/* inode number of journal file */
	s_journal_dev: u32,		/* device number of journal file */
	s_last_orphan: u32,		/* start of list of inodes to delete */
	s_hash_seed: [u32; 4],		/* HTREE hash seed */
	s_def_hash_version: u8,	/* Default hash version to use */
	s_reserved_char_pad: u8,
	s_reserved_word_pad: u16,
	s_default_mount_opts: u32,
 	s_first_meta_bg: u32, 	/* First metablock block group */
	s_reserved: [u32; 190],	/* Padding to the end of the block */
}

impl core::default::Default for Ext2SuperBlock {
    fn default() -> Ext2SuperBlock {
        unsafe { zeroed() }
    }
}

impl core::fmt::Debug for Ext2SuperBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Ext2SuperBlock\n"));
        try!(write!(f, "\tInodes count: {}\n", self.s_inodes_count));
        try!(write!(f, "\tBlocks count: {}\n", self.s_blocks_count));
        try!(write!(f, "\tReserved blocks count: {}\n", self.s_r_blocks_count));
        try!(write!(f, "\tFree blocks count: {}\n", self.s_free_blocks_count));
        try!(write!(f, "\tFree inodes count: {}\n", self.s_free_inodes_count));
        try!(write!(f, "\tFirst Data Block: {}\n", self.s_first_data_block));
        try!(write!(f, "\tBlock size: {}\n", self.s_log_block_size));
        try!(write!(f, "\tFragment size: {}\n", self.s_log_frag_size));
        try!(write!(f, "\t# Blocks per group: {}\n", self.s_blocks_per_group));
        try!(write!(f, "\t# Fragments per group: {}\n", self.s_frags_per_group));
        try!(write!(f, "\t# Inodes per group: {}\n", self.s_inodes_per_group));
        try!(write!(f, "\tMount time: {}\n", self.s_mtime));
        try!(write!(f, "\tWrite time: {}\n", self.s_wtime));
        try!(write!(f, "\tMount count: {}\n", self.s_mnt_count));
        try!(write!(f, "\tMaximal mount count: {}\n", self.s_max_mnt_count));
        try!(write!(f, "\tMagic signature: {}\n", self.s_magic));
        try!(write!(f, "\tFile system state: {}\n", self.s_state));
        try!(write!(f, "\tBehaviour when detecting errors: {}\n", self.s_errors));
        try!(write!(f, "\tminor revision level: {}\n", self.s_minor_rev_level));
        try!(write!(f, "\ttime of last check: {}\n", self.s_lastcheck));
        try!(write!(f, "\tmax. time between checks: {}\n", self.s_checkinterval));
        try!(write!(f, "\tOS: {}\n", self.s_creator_os));
        try!(write!(f, "\tRevision level: {}\n", self.s_rev_level));
        try!(write!(f, "\tDefault uid for reserved blocks: {}\n", self.s_def_resuid));
        write!(f, "\tDefault gid for reserved blocks: {}\n", self.s_def_resgid)

    }
}
