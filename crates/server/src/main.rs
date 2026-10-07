use azookey_server::TonicNamedPipeServer;
use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server::Builder as ReflectionBuilder;

use shared::proto::azookey_service_server::{AzookeyService, AzookeyServiceServer};
use shared::proto::{
    AppendTextRequest, AppendTextResponse, ClearTextRequest, ClearTextResponse, ComposingText,
    MoveCursorRequest, MoveCursorResponse, RemoveTextRequest, RemoveTextResponse,
    ShrinkTextRequest, ShrinkTextResponse, Suggestion,
};

use std::ffi::{c_char, c_int, CStr, CString};

const USE_ZENZAI: bool = true;

struct RawComposingText {
    text: String,
    cursor: i8,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct FFICandidate {
    text: *mut c_char,
    subtext: *mut c_char,
    hiragana: *mut c_char,
    corresponding_count: c_int,
}

unsafe extern "C" {
    fn Initialize(path: *const c_char, use_zenzai: bool);
    fn SetContext(context: *const c_char);
    fn AppendText(input: *const c_char, cursorPtr: *mut isize) -> *mut c_char;
    fn RemoveText(cursorPtr: *mut isize) -> *mut c_char;
    fn MoveCursor(offset: c_int, cursorPtr: *mut isize) -> *mut c_char;
    fn ShrinkText(offset: c_int) -> *mut c_char;
    fn ClearText();
    fn GetComposedText(lengthPtr: *mut isize) -> *mut *mut FFICandidate;
    fn FreeString(text: *mut c_char);
    fn FreeCandidates(candidates: *mut *mut FFICandidate, length: isize);
    fn LoadConfig();
}

unsafe fn take_string(pointer: *mut c_char) -> String {
    let result = CStr::from_ptr(pointer).to_string_lossy().into_owned();
    FreeString(pointer);
    result
}

fn initialize(path: &str) {
    unsafe {
        let path = CString::new(path).expect("CString::new failed");
        Initialize(path.as_ptr(), USE_ZENZAI);
    }
}

fn add_text(input: &str) -> RawComposingText {
    unsafe {
        let input = CString::new(input).expect("CString::new failed");
        let mut cursor: isize = 0;

        let result = AppendText(input.as_ptr(), &mut cursor);

        let text = take_string(result);

        RawComposingText {
            text,
            cursor: cursor as i8,
        }
    }
}

fn move_cursor(offset: i8) -> RawComposingText {
    unsafe {
        let offset = c_int::from(offset);
        println!("Offset: {}", offset);
        let mut cursor: isize = 0;

        let result = MoveCursor(offset, &mut cursor);

        let text = take_string(result);

        RawComposingText {
            text,
            cursor: cursor as i8,
        }
    }
}

fn remove_text() -> RawComposingText {
    unsafe {
        let mut cursor: isize = 0;

        let result = RemoveText(&mut cursor);

        let text = take_string(result);

        RawComposingText {
            text,
            cursor: cursor as i8,
        }
    }
}

fn clear_text() {
    unsafe {
        ClearText();
    }
}

fn get_composed_text() -> Vec<Suggestion> {
    let started = std::time::Instant::now();
    unsafe {
        // Swift Int has pointer width, unlike C int on 64-bit Windows.
        let mut length: isize = 0;
        let result = GetComposedText(&mut length);
        let mut suggestions = Vec::with_capacity(length as usize);

        for index in 0..length as usize {
            let candidate = (**result.add(index)).clone();
            let text = CStr::from_ptr(candidate.text)
                .to_string_lossy()
                .into_owned();
            let subtext = CStr::from_ptr(candidate.subtext)
                .to_string_lossy()
                .into_owned();
            let corresponding_count = candidate.corresponding_count;

            let suggestion = Suggestion {
                text,
                subtext,
                corresponding_count,
            };

            // check if suggestions have the same text
            if suggestions
                .iter()
                .any(|s: &Suggestion| s.text == suggestion.text)
            {
                continue;
            }
            suggestions.push(suggestion);
        }

        FreeCandidates(result, length);
        if std::env::var_os("AZOOKEY_TIMING").is_some() {
            eprintln!("conversion_ms={:.3}", started.elapsed().as_secs_f64() * 1000.0);
        }
        suggestions
    }
}

fn shrink_text(offset: i8) -> RawComposingText {
    unsafe {
        let offset = c_int::from(offset);
        let result = ShrinkText(offset);

        let text = take_string(result);

        RawComposingText {
            text,
            cursor: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct MyAzookeyService;

#[tonic::async_trait]
impl AzookeyService for MyAzookeyService {
    async fn append_text(
        &self,
        request: Request<AppendTextRequest>,
    ) -> Result<Response<AppendTextResponse>, Status> {
        let input = request.into_inner().text_to_append;
        let composing_text = add_text(&input);

        Ok(Response::new(AppendTextResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn remove_text(
        &self,
        _: Request<RemoveTextRequest>,
    ) -> Result<Response<RemoveTextResponse>, Status> {
        let composing_text = remove_text();

        Ok(Response::new(RemoveTextResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn move_cursor(
        &self,
        request: Request<MoveCursorRequest>,
    ) -> Result<Response<MoveCursorResponse>, Status> {
        let offset = request.into_inner().offset as i8;
        let composing_text = move_cursor(offset);

        Ok(Response::new(MoveCursorResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn clear_text(
        &self,
        _: Request<ClearTextRequest>,
    ) -> Result<Response<ClearTextResponse>, Status> {
        clear_text();
        Ok(Response::new(ClearTextResponse {}))
    }

    async fn shrink_text(
        &self,
        request: Request<ShrinkTextRequest>,
    ) -> Result<Response<ShrinkTextResponse>, Status> {
        let offset = request.into_inner().offset as i8;
        let composing_text = shrink_text(offset);

        Ok(Response::new(ShrinkTextResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn set_context(
        &self,
        request: Request<shared::proto::SetContextRequest>,
    ) -> Result<Response<shared::proto::SetContextResponse>, Status> {
        let context = request.into_inner().context;
        let trimmed_context = context
            .split('\r')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or_default();

        let context = CString::new(trimmed_context).expect("CString::new failed");

        unsafe { SetContext(context.as_ptr()) };
        Ok(Response::new(shared::proto::SetContextResponse {}))
    }

    async fn update_config(
        &self,
        _: Request<shared::proto::UpdateConfigRequest>,
    ) -> Result<Response<shared::proto::UpdateConfigResponse>, Status> {
        unsafe { LoadConfig() };
        Ok(Response::new(shared::proto::UpdateConfigResponse {}))
    }
}

// Swift's converter and composing text are shared mutable state. Keep FFI calls
// on this thread; none of the service handlers yield while using the converter.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AzookeyServer started");
    // get executable directory
    let current_exe = std::env::current_exe()?;
    let parent_dir = current_exe.parent().unwrap();
    initialize(parent_dir.to_str().unwrap());

    let service = MyAzookeyService::default();

    println!("AzookeyServer listening");

    Server::builder()
        .add_service(AzookeyServiceServer::new(service))
        .add_service(
            ReflectionBuilder::configure()
                .register_encoded_file_descriptor_set(shared::proto::FILE_DESCRIPTOR_SET)
                .build_v1()
                .unwrap(),
        )
        .serve_with_incoming(TonicNamedPipeServer::new("azookey_server"))
        .await?;

    Ok(())
}
