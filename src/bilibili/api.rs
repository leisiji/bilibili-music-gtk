use anyhow::{Ok, Result};

use super::data::BvidInfo;

impl BvidInfo {
    pub fn from_bvid(bvid: &str) -> Result<BvidInfo> {
        const URL_BVID_INFO: &str = "http://api.bilibili.com/x/web-interface/view?bvid=";
        let req = format!("{}{}", URL_BVID_INFO, bvid).to_string();
        let resp = ureq::get(&req).call()?.into_string()?;
        let info: BvidInfo = serde_json::from_str(resp.as_str())?;
        Ok(info)
    }
}
