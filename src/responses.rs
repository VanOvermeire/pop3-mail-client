use crate::errors::{ListError, StatError, UIDLError};

/// StatResponse is the number of messages and total size
#[derive(Debug)]
pub struct StatResponse {
    pub number_of_message: i32,
    pub total_size: i32,
}

impl TryFrom<String> for StatResponse {
    type Error = StatError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let pieces: Vec<_> = value.split(" ").collect();

        if pieces.len() == 2 {
            let number_of_message = pieces[0].parse()?;
            let total_size = pieces[1].parse()?;

            Ok(StatResponse {
                number_of_message,
                total_size,
            })
        } else {
            Err(Self::Error {
                message: format!("invalid stat response: {}", value),
            })
        }
    }
}

/// ListResponse is a list of messages
#[derive(Debug)]
pub struct ListResponse {
    pub messages: Vec<ItemResponse>,
}

impl TryFrom<String> for ListResponse {
    type Error = ListError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let messages = value.split('\n')
            .map(|v| v.replace('\r', ""))
            .filter(|v| !v.is_empty() && v != ".") // list probably ends with a single dot
            .map(|v| v.try_into())
            .collect::<Result<Vec<ItemResponse>, ListError>>()?;

        Ok(ListResponse {
            messages,
        })
    }
}

/// ItemResponse is the id and size of a message
#[derive(Debug)]
pub struct ItemResponse {
    pub message_id: i32,
    pub size: i32,
}

impl TryFrom<String> for ItemResponse {
    type Error = ListError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let pieces: Vec<_> = value.split(" ").collect();

        if pieces.len() == 2 {
            let message_id = pieces[0].parse()?;
            let size = pieces[1].parse()?;

            Ok(ItemResponse {
                message_id,
                size,
            })
        } else {
            Err(ListError {
                message: format!("invalid list item: {}", value),
            })
        }
    }
}

/// RetrieveResponse is the content of a message and its id
#[derive(Debug)]
pub struct RetrieveResponse {
    pub message_id: i32,
    pub data: String,
}

/// UIDLResponse is a list of messages with their message id and unique id
#[derive(Debug)]
pub struct UIDLResponse {
    pub messages: Vec<UIDLItem>,
}

impl TryFrom<String> for UIDLResponse {
    type Error = UIDLError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let messages = value.split('\n')
            .map(|v| v.replace('\r', ""))
            .filter(|v| !v.is_empty() && v != ".") // UIDL probably ends with a single dot
            .map(|v| v.try_into())
            .collect::<Result<Vec<UIDLItem>, UIDLError>>()?;

        Ok(UIDLResponse {
            messages,
        })
    }
}

/// UIDLItem is the id and unique id of a message
#[derive(Debug)]
pub struct UIDLItem {
    pub message_id: i32,
    pub unique_id: String,
}

impl TryFrom<String> for UIDLItem {
    type Error = UIDLError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let pieces: Vec<_> = value.split(" ").collect();

        if pieces.len() == 2 {
            let message_id = pieces[0].parse()?;
            let unique_id = pieces[1].to_string();

            Ok(UIDLItem {
                message_id,
                unique_id,
            })
        } else {
            Err(UIDLError {
                message: format!("invalid UIDL item: {}", value),
            })
        }
    }
}

/// TopResponse is the id of the message, the number of lines that top had to return, and the data of those lines
#[derive(Debug)]
pub struct TopResponse {
    pub message_id: i32,
    pub number_of_lines: i32,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_response_try_from() {
        let actual: StatResponse = "2 12345".to_string().try_into().unwrap();

        assert_eq!(actual.number_of_message, 2);
        assert_eq!(actual.total_size, 12345);
    }

    #[test]
    fn test_item_response_try_from() {
        let actual: ItemResponse = "2 12345".to_string().try_into().unwrap();

        assert_eq!(actual.message_id, 2);
        assert_eq!(actual.size, 12345);
    }

    #[test]
    fn test_item_response_try_from_fails_for_response_with_no_spaces() {
        let actual: Result<ItemResponse, ListError> = "invalid".to_string().try_into();

        assert!(actual.is_err())
    }

    #[test]
    fn test_item_response_try_from_fails_for_response_without_numbers() {
        let actual: Result<ItemResponse, ListError> = "a bcd".to_string().try_into();

        assert!(actual.is_err())
    }

    #[test]
    fn test_list_response_try_from() {
        let actual: ListResponse = "1 12345\n2 2345".to_string().try_into().unwrap();

        assert_eq!(actual.messages.len(), 2);
        assert_eq!(actual.messages[0].message_id, 1);
        assert_eq!(actual.messages[0].size, 12345);
        assert_eq!(actual.messages[1].message_id, 2);
        assert_eq!(actual.messages[1].size, 2345);
    }

    #[test]
    fn test_list_response_try_from_ending_period() {
        let actual: ListResponse = "1 12345\n2 2345\n.".to_string().try_into().unwrap();

        assert_eq!(actual.messages.len(), 2);
        assert_eq!(actual.messages[0].message_id, 1);
        assert_eq!(actual.messages[0].size, 12345);
        assert_eq!(actual.messages[1].message_id, 2);
        assert_eq!(actual.messages[1].size, 2345);
    }

    #[test]
    fn test_list_response_try_from_with_carriage_return() {
        let actual: ListResponse = "1 12345\r\n2 2345".to_string().try_into().unwrap();

        assert_eq!(actual.messages.len(), 2);
        assert_eq!(actual.messages[0].message_id, 1);
        assert_eq!(actual.messages[0].size, 12345);
        assert_eq!(actual.messages[1].message_id, 2);
        assert_eq!(actual.messages[1].size, 2345);
    }

    #[test]
    fn test_list_response_try_from_with_returnsat_end() {
        let actual: ListResponse = "1 12345\r\n2 2345\r\n".to_string().try_into().unwrap();

        assert_eq!(actual.messages.len(), 2);
        assert_eq!(actual.messages[0].message_id, 1);
        assert_eq!(actual.messages[0].size, 12345);
        assert_eq!(actual.messages[1].message_id, 2);
        assert_eq!(actual.messages[1].size, 2345);
    }

    #[test]
    fn test_list_response_try_from_invalid() {
        let actual: Result<ListResponse, ListError> = "1\r\n2".to_string().try_into();

        assert!(actual.is_err());
    }

    #[test]
    fn test_uidl_response_try_from_with_returns_at_end() {
        let actual: UIDLResponse = "1 whqtswO00WBw418f9t5JxYwZ\r\n2 QhdPYR:00WBw1Ph7x7\r\n".to_string().try_into().unwrap();

        assert!(actual.messages.len() == 2);
        assert_eq!(actual.messages[0].message_id, 1);
        assert_eq!(actual.messages[0].unique_id, "whqtswO00WBw418f9t5JxYwZ".to_string());
        assert_eq!(actual.messages[1].message_id, 2);
        assert_eq!(actual.messages[1].unique_id, "QhdPYR:00WBw1Ph7x7".to_string());
    }

    #[test]
    fn test_uidl_response_try_from() {
        let actual: UIDLResponse = "1 whqtswO00WBw418f9t5JxYwZ\r\n2 QhdPYR:00WBw1Ph7x7".to_string().try_into().unwrap();

        assert!(actual.messages.len() == 2);
        assert_eq!(actual.messages[0].message_id, 1);
        assert_eq!(actual.messages[0].unique_id, "whqtswO00WBw418f9t5JxYwZ".to_string());
        assert_eq!(actual.messages[1].message_id, 2);
        assert_eq!(actual.messages[1].unique_id, "QhdPYR:00WBw1Ph7x7".to_string());
    }
}