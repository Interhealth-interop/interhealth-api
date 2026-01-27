use serde_json::{json, Value};

pub fn get_template() -> Value {
    json!({
        "fullUrl": "",
        "resource": {
            "resourceType": "",
            "meta": {
                "tag": [
                    {
                        "system": "http://interop.interhealth.com.br/NamingSystem/client-id",
                        "code": ""
                    },
                    {
                        "system": "http://interop.interhealth.com.br/NamingSystem/data-provider",
                        "code": ""
                    },
                    {
                        "system": "http://interop.interhealth.com.br/NamingSystem/data-type",
                        "code": ""
                    }
                ]
            }
        },
        "request": {
            "method": "POST",
            "url": "",
            "ifNoneExist": ""
        }
    })
}