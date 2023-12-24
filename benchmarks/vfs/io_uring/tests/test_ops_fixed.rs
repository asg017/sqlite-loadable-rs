#[cfg(test)]
mod tests {
    use _iouringvfs::ops::OpsFixed;
    use std::ffi::CString;
    use std::io::Result;
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;
    use tempfile::TempDir;

    fn create_file(dir: &TempDir, file_name: &str, write: Option<&[u8]>) -> CString {
        let path_buf = dir.path().join(file_name);
        let path = CString::new(path_buf.as_os_str().as_bytes()).expect("bad path");
        if let Some(b) = write {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path_buf)
                .expect("failed to create new file");
            let _ = file.write(b);
        }
        path
    }

    #[test]
    fn test_open_and_close_file() -> Result<()> {
        let dir = tempfile::tempdir().expect("bad dir");
        let path = create_file(&dir, "main.db-journal", None);
        let mut ops = OpsFixed::new(path.as_ptr() as _, 16);

        // Check if the operation was successful
        ops.open_file()?;

        unsafe {
            ops.o_close()?;
        }

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_create_write_close_file() -> Result<()> {
        let dir = tempfile::tempdir().expect("bad dir");
        let path = create_file(&dir, "main.db-journal", None);
        let mut ops = OpsFixed::new(path.as_ptr() as _, 16);

        ops.open_file()?;

        // Write data to the file
        let data_to_write = b"Hello, World!";
        unsafe { ops.o_write(data_to_write.as_ptr() as _, 0, 13) }?;

        unsafe {
            ops.o_close()?;
        }

        Ok(())
    }

    #[test]
    fn test_read() -> Result<()> {
        let data_to_write = b"Hello, World!";

        let dir = tempfile::tempdir().expect("bad dir");
        let path = create_file(&dir, "main.db-journal", Some(data_to_write));
        let mut ops = OpsFixed::new(path.as_ptr() as _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Read the file
        let mut buf: [u8; 13] = [0; 13];
        unsafe {
            ops.o_read(0, 13, buf.as_mut_ptr() as _)?;
        }

        // Check if the data read matches what was written
        assert_eq!(buf[..], data_to_write[..]);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_write() -> Result<()> {
        let tmpfile = tempfile::NamedTempFile::new()?;
        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;

        let mut ops = OpsFixed::new(file_path.as_ptr() as _, 16);

        ops.open_file()?;

        let data_to_write = b"Hello, World!";
        unsafe {
            ops.o_write(
                data_to_write.as_ptr() as _,
                0,
                data_to_write.len().try_into().unwrap(),
            )
        }?;

        let file = tmpfile.as_file();

        assert_eq!(file.metadata()?.len(), data_to_write.len() as u64);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_write_then_read() -> Result<()> {
        // Create a temporary file for testing
        let dir = tempfile::tempdir().expect("bad dir");
        let path = create_file(&dir, "main.db-journal", None);
        let mut ops = OpsFixed::new(path.as_ptr() as _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Write data to the file
        let data_to_write = b"Hello, World!";
        let mut buf: [u8; 13] = [0; 13];
        unsafe {
            ops.o_write(data_to_write.as_ptr() as _, 0, 13)?;
            ops.o_fsync(0)?;
            ops.o_read(0, 13, buf.as_mut_ptr() as _)?;
        }

        // Check if the data read matches what was written
        assert_eq!(buf[..], data_to_write[..]);

        Ok(())
    }

    #[test]
    fn test_file_size() -> Result<()> {
        let data_to_write = b"Hello, World!";

        let dir = tempfile::tempdir().expect("bad dir");
        let path = create_file(&dir, "main.db-journal", Some(data_to_write));

        let mut ops = OpsFixed::new(path.as_ptr() as _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Get the current file size
        let mut file_size: u64 = 0;
        unsafe {
            ops.o_file_size(&mut file_size)?;
        }

        assert_eq!(file_size, 13);

        Ok(())
    }

    #[test]
    fn test_truncate_then_compare_file_size() -> Result<()> {
        // Create a temporary file for testing
        let mut tmpfile = tempfile::NamedTempFile::new()?;
        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;
        let mut ops = OpsFixed::new(file_path.as_ptr() as _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Write some data to the file
        let data_to_write = b"Hello, World!";
        tmpfile.write(data_to_write)?;

        // Truncate the file to a smaller size
        let new_size = 5; // Set the new size to 5 bytes
        unsafe {
            ops.o_truncate(new_size)?;
        }

        // Get the current file size
        let mut file_size: u64 = 0;
        unsafe {
            ops.o_file_size(&mut file_size)?;
        }

        // Check if the file size matches the expected size
        assert_eq!(file_size, new_size as u64);

        // Cleanup
        tmpfile.close()?;

        Ok(())
    }
}
