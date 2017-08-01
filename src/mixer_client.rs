extern crate grpc;
extern crate futures;



use std::env;
use std::mem;
use std::ptr;
use std::str;
use std::slice;
use std::collections::HashMap;
use service_grpc::MixerClient;
use report::ReportRequest;
use attributes::Attributes;
use attributes::StringMap;
use service_grpc::Mixer;

use bindings::ngx_http_request_s;
use nginx_http::request_iterator;
use nginx_http::log;
use bindings::ngx_str_t;


static REQUEST_HEADER: i32 = 0;
static TARGET_SERVICE: i32 = 1;


/**
  filter nginx 
 */
/*
#[no_mangle] 
public extern fn ngx_int_t ngx_http_istio_mixer_filter(request: *const ngx_http_request_s) -> ngx_int_t {
{

    log(&format!("start invoking istio mixer filter");

    ngx_http_mixer_main_conf_t *conf = *rngx_http_istio_mixer_module);

    ngx_log_error(NGX_LOG_ERR, ngx_cycle->log, 0, "using server: %*s",conf->mixer_server.len,conf->mixer_server.data);

    // invoke mix client
    mixer_client(r);

    ngx_log_error(NGX_LOG_ERR, ngx_cycle->log, 0, "finish calling istio filter");

    return ngx_http_next_header_filter(r);

} */




#[no_mangle] 
pub extern fn mixer_client(request: *const ngx_http_request_s,ng_server: *const ngx_str_t,port: u32) -> *const u8 {

    let server = unsafe { *ng_server } ;
    let server_name = server.to_str()  ;

     log(&format!("server port {}",port));

    let client = MixerClient::new_plain(server_name, 9091, Default::default()).expect("init");

    let mut requests = Vec::new();
    let mut req = ReportRequest::new();
    let mut attr = Attributes::new();
    //attr.set_string_attributes("")
    req.set_request_index(0);

    // set up attribute dictionary 
    let mut dictValues: HashMap<i32,String> = HashMap::new();
    dictValues.insert(REQUEST_HEADER,String::from("request.headers"));
    dictValues.insert(TARGET_SERVICE,String::from("target.service"));


    let mut stringMapValues: HashMap<i32,StringMap> = HashMap::new();
    let stringMap = process_request_attribute(request,&mut dictValues);
    stringMapValues.insert(REQUEST_HEADER,stringMap);
    
    attr.set_dictionary(dictValues);
    attr.set_stringMap_attributes(stringMapValues);
    
  
    req.set_attribute_update(attr);


    requests.push(req);


    let resp = client.report(grpc::RequestOptions::new(), grpc::StreamingRequest::iter(requests));

    resp.wait_drop_metadata().count();

    "Hello, world!\0".as_ptr()
}


// find string index from dictionary
fn string_index(value: &str, dictValues: &HashMap<i32,String>) -> Option<i32> {

    log(&format!("checking if value: {} exists in dictionary",value));
    for( index ,dictValue) in dictValues  {
        log(&format!("comparing existing dictionary: {}",&dictValue));
        if value == dictValue {
            log(&format!("matched existing value"));
            return Some(index.abs());
        }
    }
    log(&format!("did not match"));
    return None;
}

// process request attribute,
// loop over request header and add to dictionary
// then use that to build new string map
fn process_request_attribute(request: *const ngx_http_request_s, dictValues: &mut HashMap<i32,String>) -> StringMap {


    let mut map: HashMap<i32,String> = HashMap::new();
  

    for (name,value) in request_iterator(request)  {
        log(&format!("header name: {}, value: {}",&name,&value));

        let result = string_index(&value,dictValues);

        match result  {
            Some(index) =>  {
                map.insert(index,value);
                log(&format!("match existing index: {}",index));
            },
            None =>  {
                let newIndex = dictValues.len() as i32  + 1;
                dictValues.insert(newIndex,value.clone());
                map.insert(newIndex,value.clone());
                log(&format!("adding to string map index: {}, value: {}",newIndex,&value));
            },
            
        }
        
    }

    let mut stringMap = StringMap::new();
    stringMap.set_map(map);
    return stringMap;

}


