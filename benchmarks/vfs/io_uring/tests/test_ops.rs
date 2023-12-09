use std::ffi::CString;
use std::io::Result;
use std::os::raw::c_void;

#[cfg(test)]
mod tests {
    use super::*;
    use _iouringvfs::ops::Ops;
    use std::io::Write;

    #[test]
    fn test_open_and_close_file() -> Result<()> {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new()?;
        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;
        let mut ops = Ops::new(file_path.as_ptr() as *const _, 16);

        // Perform the open operation
        let result = ops.open_file();

        // Check if the operation was successful
        assert!(result.is_ok());

        unsafe {
            ops.o_close()?;
        }

        // Cleanup
        tmpfile.close()?;

        Ok(())
    }

    #[test]
    fn test_create_write_close_file() -> Result<()> {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new()?;

        let mut new_file_name = tmpfile.path().to_string_lossy().to_string();
        new_file_name.push_str("-journal");

        let mut ops = Ops::new(new_file_name.as_ptr() as *const _, 16);

        // Perform the open operation to create the file
        let result = ops.open_file();

        // Check if the operation was successful
        assert!(result.is_ok());

        // Write data to the file
        let data_to_write = b"Hello, World!";
        unsafe { ops.o_write(data_to_write.as_ptr() as *const c_void, 0, 13) }?;

        // Check if the operation was successful
        assert!(result.is_ok());

        unsafe {
            ops.o_close()?;
            std::fs::remove_file(new_file_name)?;
        }

        // Cleanup
        tmpfile.close()?;

        Ok(())
    }

    #[test]
    fn test_read() -> Result<()> {
        // Create a temporary file for testing
        let mut tmpfile = tempfile::NamedTempFile::new()?;

        let data_to_write = b"Hello, World!";
        tmpfile.write(data_to_write)?;

        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;
        let mut ops = Ops::new(file_path.as_ptr() as *const _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Read the file
        let mut buf: [u8; 13] = [0; 13];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        unsafe {
            ops.o_read(0, 13, buf_ptr)?;
        }

        // Check if the data read matches what was written
        assert_eq!(buf[..], data_to_write[..]);

        // Cleanup
        tmpfile.close()?;

        Ok(())
    }

    #[test]
    fn test_write_then_read() -> Result<()> {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new()?;
        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;
        let mut ops = Ops::new(file_path.as_ptr() as *const _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Write data to the file
        let data_to_write = b"Hello, World!";
        let mut buf: [u8; 13] = [0; 13];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        unsafe {
            ops.o_write(data_to_write.as_ptr() as *const c_void, 0, 13)?;
            ops.o_read(0, 13, buf_ptr)?;
        }

        // Check if the data read matches what was written
        assert_eq!(buf[..], data_to_write[..]);

        // Cleanup
        tmpfile.close()?;

        Ok(())
    }

    #[test]
    fn test_file_size() -> Result<()> {
        // Create a temporary file for testing
        let mut tmpfile = tempfile::NamedTempFile::new()?;
        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;

        let data_to_write = b"Hello, World!";
        tmpfile.write(data_to_write)?;

        let mut ops = Ops::new(file_path.as_ptr() as *const _, 16);

        // Perform the open operation
        ops.open_file()?;

        // Get the current file size
        let mut file_size: u64 = 0;
        unsafe {
            ops.o_file_size(&mut file_size)?;
        }

        assert_eq!(file_size, 13);

        // Cleanup
        tmpfile.close()?;

        Ok(())
    }

    #[test]
    fn test_truncate_then_compare_file_size() -> Result<()> {
        // Create a temporary file for testing
        let mut tmpfile = tempfile::NamedTempFile::new()?;
        let file_path = CString::new(tmpfile.path().to_string_lossy().to_string())?;
        let mut ops = Ops::new(file_path.as_ptr() as *const _, 16);


        // Perform the open operation
        ops.open_file()?;
        // let raw_fd = tmpfile.as_raw_fd();
        // ops.set_fd(raw_fd);

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
