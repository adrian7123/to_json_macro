#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use to_json_macro::ToJson;

    use bson::{oid::ObjectId, DateTime};

    #[derive(Clone, ToJson, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Test {
        id: ObjectId,
        name: String,
        pub created_at: DateTime,
        pub updated_at: DateTime,
    }

    #[derive(ToJson, Serialize, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct MyStruct {
        #[serde(rename = "_id")]
        _id: ObjectId,
        #[serde(rename = "testId")]
        test_id: Option<ObjectId>,
        #[serde(rename = "testId2")]
        test_id2: Option<ObjectId>,
        name: String,
        value: i32,
        #[serde(rename = "dateTime")]
        date_time: DateTime,
        #[serde(rename = "dateTimeOption")]
        date_time_option: Option<DateTime>,
        #[json]
        two: Two,
        #[serde(rename = "twoOpt")]
        #[json]
        two_opt: Option<Two>,
        #[serde(rename = "twoVec")]
        #[json]
        two_vec: Vec<Two>,
        str_vec: Vec<String>,
        vec_object_id: Vec<ObjectId>,
    }

    #[derive(ToJson, Serialize, Clone, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Two {
        #[serde(rename = "_id")]
        id: ObjectId,
        test_id2: Option<ObjectId>,
        #[json]
        pub register_status: Option<RegisterStatus>,
        #[json]
        pub register_status2: Option<RegisterStatus>,
    }

    #[derive(ToJson, Serialize, Clone, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[allow(dead_code)]
    enum RegisterStatus {
        Pending,
        Complete,
    }

    #[test]
    fn test_to_json() {
        let my_struct = MyStruct {
            vec_object_id: vec![ObjectId::new(), ObjectId::new()],
            _id: ObjectId::new(),
            name: "test".to_string(),
            value: 42,
            date_time: DateTime::now(),
            test_id: Some(ObjectId::new()),
            test_id2: None,
            date_time_option: Some(DateTime::now()),
            two: Two {
                id: ObjectId::new(),
                test_id2: Some(ObjectId::new()),
                register_status: Some(RegisterStatus::Pending),
                register_status2: None,
            },
            two_opt: None,
            two_vec: vec![
                Two {
                    id: ObjectId::new(),
                    test_id2: Some(ObjectId::new()),
                    register_status: Some(RegisterStatus::Pending),
                    register_status2: None,
                },
                Two {
                    id: ObjectId::new(),
                    test_id2: Some(ObjectId::new()),
                    register_status: Some(RegisterStatus::Pending),
                    register_status2: None,
                },
            ],
            str_vec: vec!["test".to_string(), "test2".to_string()],
        };

        print!(
            "{}\n",
            serde_json::to_string_pretty(&my_struct.to_json())
                .expect("Failed to serialize to JSON")
        );

        let json_value = my_struct.to_json();

        assert_eq!(json_value["_id"], json!(my_struct._id.to_string()));
        assert_eq!(json_value["name"], json!(my_struct.name));
        assert_eq!(json_value["value"], json!(my_struct.value));
    }
}
