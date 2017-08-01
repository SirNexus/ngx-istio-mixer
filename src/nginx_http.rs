
extern crate libc;


use std::str;
use std::slice;
use std::ffi::CString;


use bindings::ngx_http_request_s;
use bindings::ngx_list_part_t;
use bindings::ngx_table_elt_t;
use bindings::ngx_uint_t;
use bindings::ngx_str_t;
use bindings::ngx_log_error_core;
use bindings::NGX_LOG_ERR;
use bindings::ngx_cycle;


impl ngx_str_t  {
    // convert nginx string to str slice
    pub fn to_str(&self) -> &str  {

        unsafe {
            let slice = slice::from_raw_parts(self.data,self.len) ;
            return str::from_utf8(slice).unwrap();
        }            
   
    }

    // get string 
    pub fn to_string(&self) -> String  {
        return String::from(self.to_str());
    }
}


pub struct HttpRequestIterator {

    done: bool ,
    part: *const ngx_list_part_t,
    h: *const ngx_table_elt_t,
    i: ngx_uint_t
}


// create new http request iterator 
pub fn request_iterator(request: *const ngx_http_request_s) -> HttpRequestIterator  {

    unsafe {
        let  part: *const ngx_list_part_t  = &(*request).headers_in.headers.part ;

        HttpRequestIterator  {
            done: false,
            part: part,
            h: (*part).elts as *const ngx_table_elt_t,
            i: 0
        }
    }
    

}

impl Iterator for HttpRequestIterator  {

    // type Item = (&str,&str);
    // TODO: try to use str instead of string

    type Item = (String,String);
    
    fn next(&mut self) -> Option<Self::Item> {

        unsafe {
            if self.done  {
                return None;
            } else {
                if self.i >= (*self.part).nelts  {
                        if (*self.part).next.is_null() {
                            self.done = true;
                            return None
                        }

                        // loop back
                        self.part = (*self.part).next;
                        self.h = (*self.part).elts as *mut ngx_table_elt_t;
                        self.i = 0;
                }

                let header: *const ngx_table_elt_t = self.h.offset(self.i as isize);

                let header_name: ngx_str_t = (*header).key;   
                    
                let header_value: ngx_str_t = (*header).value;
               
                self.i = self.i + 1;

                return Some( (header_name.to_string(),header_value.to_string()) ) ;

            }
        }
    
    }

}

// log message
pub fn log(message: &str)  {

    unsafe {
          ngx_log_error_core(NGX_LOG_ERR as usize, (*ngx_cycle).log, 0, message.as_ptr() as *const ::std::os::raw::c_char,message.len());
    }
     
}
