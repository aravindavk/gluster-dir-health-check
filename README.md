# Gluster Directories Health Check

A tool to detect stale or boken directories in Gluster bricks. This tool can detect following issues

**No GFID**: If a directory in brick backend doesn't have `trusted.gfid` xattr  
**No Parent GFID**: Parent directory doesn't have a `trusted.gfid` xattr  
**No Symlink**: Gluster maintains a symlink in `$BRICK/.glusterfs` directory for each directory, If Symlink file not present for a directory  
**Wrong Symlink**: If symlink exists but linked to different directory  
**Invalid GFID**: Invalid data in `trusted.gfid` xattr  
**Invalid Parent GFID**: Invalid data in `trusted.gfid` xattr of parent dir  

## Install

If Rust is installed in your machine then run the following

    cargo install --git https://github.com/aravindavk/gluster_dir_health_check

## Usage

    gluster_dir_health_check <brick_path>

Example:

    gluster_dir_health_check /bricks/b1
    gluster_dir_health_check /bricks/b1 > ~/bricks_b1_dirs_status.txt

## Output

Example output,

    STATUS   DESCRIPTION   GFID                                 PGFID                                PATH
    -------- ------------- ------------------------------------ ------------------------------------ ---------------------
    [OK    ]               0b875a91-1a51-42fc-b68a-83c9248bdbb8 00000000-0000-0000-0000-000000000001 /bricks/b1/d1
    [OK    ]               a6e870e4-4376-493c-96ca-678f0e1d01fe 00000000-0000-0000-0000-000000000001 /bricks/b1/n1
    [NOT OK] WRONG SYMLINK a6e870e4-4376-493c-96ca-678f0e1d01fe 00000000-0000-0000-0000-000000000001 /bricks/b1/h1
    [OK    ]               bac0c3fb-26c8-403b-85ab-e344b7c10011 00000000-0000-0000-0000-000000000001 /bricks/b1/md1
    [OK    ]               9bef6669-ac60-48c0-8599-bd6dd1909957 bac0c3fb-26c8-403b-85ab-e344b7c10011 /bricks/b1/md1/sd1

Above output shows GFID `a6e870e4-4376-493c-96ca-678f0e1d01fe` is associated with two directories `/bricks/b1/n1` and `/bricks/b1/h1`. But backend symlink is with associated with `/bricks/b1/n1`
