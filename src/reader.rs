use std::io::Read;

const READ_BUFFER_SIZE: usize = 512;
const READ_ALL_BUFFER_SIZE: usize = 2048; // bigger calls can probably use a bigger buffer? depends on how much data we get in one go though

const PERIOD_SURROUNDED_BY_NEWLINE: [u8; 3] = [10, 46, 10];
const PERIOD_SURROUNDED_BY_CARRIAGE_RETURN_AND_NEWLINE: [u8; 5] = [13, 10, 46, 13, 10];

const ZERO: u8 = 0;
const NEWLINE: u8 = 10;
const HYPHEN: u8 = 45;

const OK_RESPONSE_START: &'static str = "+OK";
const ERR_RESPONSE_START: &'static str = "-ERR";

pub fn read_response(reader: &mut impl Read) -> Result<String, String> {
    let response = read(reader);
    translate_string_response(response)
}

pub fn read_multi_response(reader: &mut impl Read) -> Result<String, String> {
    let response = read_all(reader);
    translate_string_response(response)
}

fn translate_string_response(response: String) -> Result<String, String> {
    if response.starts_with(OK_RESPONSE_START) {
        Ok(response.replace(OK_RESPONSE_START, "").trim().to_string())
    } else if response.starts_with(ERR_RESPONSE_START) {
        Err(response.replace(ERR_RESPONSE_START, "").replace("\r\n", "").trim().to_string())
    } else {
        Err(format!("unexpected response: {response}"))
    }
}

fn read_all(reader: &mut impl Read) -> String {
    let mut line_buffer: Vec<u8> = Vec::new();

    // we should always get at least 3 u8s, since we have an OK/ERR + \r\n
    while line_buffer.len() < 3 || (!ends_with_sole_period_and_newline(&line_buffer) && !is_err(&line_buffer)) {
        let mut byte_buffer = [0; READ_ALL_BUFFER_SIZE];
        reader.read(&mut byte_buffer).expect("reading to work");
        line_buffer.extend_from_slice(&byte_buffer);
        // our buffer might be too long - remove 0 content
        line_buffer = line_buffer.into_iter().filter(|v| v != &ZERO).collect();
    }
    String::from_utf8_lossy(&line_buffer).into_owned()
}

fn is_err(line_buffer: &Vec<u8>) -> bool {
    line_buffer[0] == HYPHEN
}

fn ends_with_sole_period_and_newline(line_buffer: &Vec<u8>) -> bool {
    let selection = &line_buffer[line_buffer.len() - 3..line_buffer.len()];
    let second_selection = &line_buffer[line_buffer.len() - 5..line_buffer.len()];
    selection == PERIOD_SURROUNDED_BY_NEWLINE || second_selection == PERIOD_SURROUNDED_BY_CARRIAGE_RETURN_AND_NEWLINE
}

fn read(reader: &mut impl Read) -> String {
    let mut line_buffer: Vec<u8> = Vec::new();

    while line_buffer.len() < 2 || line_buffer[line_buffer.len() - 1] != NEWLINE {
        let mut byte_buffer = [0; READ_BUFFER_SIZE];
        reader.read(&mut byte_buffer).expect("reading to work");
        line_buffer.extend_from_slice(&byte_buffer);
        // our buffer might be too long - remove 0 content
        // could also optimize by reading shorter stuff for commands that only have something like 'OK' as relevant info
        line_buffer = line_buffer.into_iter().filter(|v| v != &ZERO).collect();
    }
    String::from_utf8_lossy(&line_buffer).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_response_ok_result() {
        let data = b"+OK Hello \n";
        let mut slice: &[u8] = data.as_ref();

        let actual = read_response(&mut slice).unwrap();

        assert_eq!(actual, "Hello".to_string());
    }

    #[test]
    fn test_read_response_err_result() {
        let data = b"-ERR an error\r\n";
        let mut slice: &[u8] = data.as_ref();

        let actual = read_response(&mut slice);

        assert!(actual.is_err());
        assert_eq!(actual.err().unwrap(), "an error".to_string());
    }

    #[test]
    fn test_read_response_unknown_result() {
        let data = b"Something unexpected\n";
        let mut slice: &[u8] = data.as_ref();

        let actual = read_response(&mut slice);

        assert!(actual.is_err());
        assert_eq!(actual.err().unwrap(), "unexpected response: Something unexpected\n".to_string());
    }

    #[test]
    fn test_read_multi_response_ok_result_with_carriage_return() {
        let data = b"+OK Some \nThings \r\n.\r\n";
        let mut slice: &[u8] = data.as_ref();

        let actual = read_multi_response(&mut slice).unwrap();

        assert_eq!(actual, "Some \nThings \r\n.".to_string());
    }

    #[test]
    fn test_read_multi_response_ok_result_no_carriage_return() {
        let data = b"+OK Some \nThings\n.\n";
        let mut slice: &[u8] = data.as_ref();

        let actual = read_multi_response(&mut slice).unwrap();

        assert_eq!(actual, "Some \nThings\n.".to_string());
    }

    #[test]
    fn test_read_multi_response_err_result() {
        let data = b"-ERR Protocol error \n";
        let mut slice: &[u8] = data.as_ref();

        let actual = read_multi_response(&mut slice);

        assert_eq!(actual.err().unwrap(), "Protocol error".to_string());
    }
}