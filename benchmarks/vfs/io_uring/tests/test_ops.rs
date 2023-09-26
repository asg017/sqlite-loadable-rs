use sqlite3_vfs_io_uring_rs::ops::Ops;
use std::ffi::CString;
use std::os::raw::c_void;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_file() {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        let file_path = CString::new(tmpfile.path().to_str().unwrap()).unwrap();
        let mut ops = Ops::new(file_path.clone(), 16);

        // Perform the open operation
        let result = ops.open_file();

        // Check if the operation was successful
        assert!(result.is_ok());

        // Cleanup
        tmpfile.close().unwrap();
    }

    #[test]
    fn test_read_and_write() {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        let file_path = CString::new(tmpfile.path().to_str().unwrap()).unwrap();
        let mut ops = Ops::new(file_path.clone(), 16);

        // Perform the open operation
        ops.open_file().unwrap();

        // Write data to the file
        let data_to_write = b"Hello, World!";
        let mut buf: [u8;13] = [0; 13];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        unsafe {
            ops.o_write(data_to_write.as_ptr() as *const c_void, 0, 13).unwrap();
            ops.o_read(0, 13, buf_ptr).unwrap();
        }

        // // Check if the data read matches what was written
        for i in 0..13 {
            assert_eq!(buf[i], data_to_write[i]);
        }

        // Cleanup
        tmpfile.close().unwrap();
    }

    #[test]
    fn test_file_size() {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        let file_path = CString::new(tmpfile.path().to_str().unwrap()).unwrap();
        let mut ops = Ops::new(file_path.clone(), 16);
    
        // Perform the open operation
        ops.open_file().unwrap();
    
        // Get the current file size
        let mut file_size: u64 = 0;
        unsafe {
            ops.o_file_size(&mut file_size).unwrap();
        }
    
        // Expected file size is 0 since the file is empty
        assert_eq!(file_size, 0);
    
        // Cleanup
        tmpfile.close().unwrap();
    }

    #[test]
    fn test_truncate() {
        // Create a temporary file for testing
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        let file_path = CString::new(tmpfile.path().to_str().unwrap()).unwrap();
        let mut ops = Ops::new(file_path.clone(), 16);

        // Perform the open operation
        ops.open_file().unwrap();

        // Write some data to the file
        let data_to_write = b"Hello, World!";
        let data_len = data_to_write.len() as i64;
        unsafe {
            ops.o_write(data_to_write.as_ptr() as *const c_void, 0, data_len as u32).unwrap();
        }

        // Truncate the file to a smaller size
        let new_size = 5; // Set the new size to 5 bytes
        ops.o_truncate(new_size).unwrap();

        // Get the current file size
        let mut file_size: u64 = 0;
        unsafe {
            ops.o_file_size(&mut file_size).unwrap();
        }

        // Check if the file size matches the expected size
        assert_eq!(file_size, new_size as u64);

        // Cleanup
        tmpfile.close().unwrap();
    }
}
