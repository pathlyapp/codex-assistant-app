use serde::Serialize;
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Read},
    time::Duration,
};
use url::Url;

const MAX_MODELS_RESPONSE_BYTES: u64 = 1024 * 1024;
const MAX_RESPONSES_PROBE_BYTES: u64 = 1024 * 1024;
const MAX_MODEL_COUNT: usize = 2_000;
const PROBE_INPUT: &str = "Return OK.";
const PROBE_MAX_OUTPUT_TOKENS: u16 = 16;
const MODELS_TIMEOUT: Duration = Duration::from_secs(12);
const RESPONSES_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Clone, Debug)]
pub struct ModelEntry {
    pub id: String,
    /// OpenRouter 等服务在 /models 里声明的输入模态（如 text、image、video）。
    /// 服务未提供该信息时为 None。
    pub input_modalities: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsesProbeResult {
    pub protocol: String,
    pub model: String,
    pub request_id: Option<String>,
    pub completed: bool,
}

pub struct RouterClient {
    gateway: String,
    bearer: Option<String>,
    agent: ureq::Agent,
}

impl RouterClient {
    pub fn new(gateway: &str, bearer: Option<&str>) -> Self {
        Self {
            gateway: gateway.trim_end_matches('/').to_string(),
            bearer: bearer.map(str::to_string),
            agent: ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(4))
                .timeout_read(RESPONSES_TIMEOUT)
                .timeout_write(Duration::from_secs(8))
                .build(),
        }
    }

    pub fn fetch_models(&self) -> Result<Vec<String>, String> {
        Ok(self
            .fetch_model_entries()?
            .into_iter()
            .map(|entry| entry.id)
            .collect())
    }

    pub fn fetch_model_entries(&self) -> Result<Vec<ModelEntry>, String> {
        let endpoint = self.endpoint("models");
        let mut request = self
            .agent
            .get(&endpoint)
            .timeout(MODELS_TIMEOUT)
            .set("Accept", "application/json");
        if let Some(token) = self.bearer.as_deref() {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        let response = request
            .call()
            .map_err(|error| http_error("GET /models", &self.gateway, error))?;
        let body = read_limited(response.into_reader(), MAX_MODELS_RESPONSE_BYTES, "/models")?;
        parse_models_response(&body, &self.gateway)
    }

    pub fn probe_responses(&self, model: &str) -> Result<ResponsesProbeResult, String> {
        let endpoint = self.endpoint("responses");
        let mut request = self
            .agent
            .post(&endpoint)
            .timeout(RESPONSES_TIMEOUT)
            .set("Accept", "text/event-stream, application/json")
            .set("Content-Type", "application/json");
        if let Some(token) = self.bearer.as_deref() {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
        let response = request
            .send_json(probe_request(model))
            .map_err(|error| http_error("POST /responses", &self.gateway, error))?;
        let request_id = response
            .header("x-request-id")
            .or_else(|| response.header("request-id"))
            .map(str::to_string);
        let content_type = response
            .header("Content-Type")
            .unwrap_or_default()
            .to_ascii_lowercase();
        if content_type.contains("text/event-stream") {
            parse_sse_probe(response.into_reader(), model, request_id)
        } else {
            let body = read_limited(
                response.into_reader(),
                MAX_RESPONSES_PROBE_BYTES,
                "/responses",
            )?;
            parse_json_probe(&body, model, request_id)
        }
    }

    fn endpoint(&self, resource: &str) -> String {
        format!("{}/{resource}", self.gateway)
    }
}

fn probe_request(model: &str) -> Value {
    json!({
        "model": model,
        "input": PROBE_INPUT,
        "stream": true,
        "max_output_tokens": PROBE_MAX_OUTPUT_TOKENS,
    })
}

fn parse_models_response(body: &[u8], gateway: &str) -> Result<Vec<ModelEntry>, String> {
    let payload: Value =
        serde_json::from_slice(body).map_err(|_| "/models 未返回有效 JSON".to_string())?;
    let mut models = payload
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .take(MAX_MODEL_COUNT + 1)
                .filter_map(parse_model_entry)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if models.len() > MAX_MODEL_COUNT {
        return Err(format!(
            "Router /models 返回超过 {MAX_MODEL_COUNT} 个模型，已停止处理"
        ));
    }
    models.sort_by(|left, right| left.id.cmp(&right.id));
    models.dedup_by(|left, right| left.id == right.id);
    if models.is_empty() {
        if is_local_ollama_gateway(gateway) {
            return Err(
                "Ollama 已启动，但尚未返回模型。请先运行 ollama pull <模型名> 下载至少一个模型"
                    .to_string(),
            );
        }
        return Err("Router /models 没有返回可用模型".to_string());
    }
    Ok(models)
}

fn parse_model_entry(item: &Value) -> Option<ModelEntry> {
    let id = item
        .get("id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())?;
    let input_modalities = item
        .get("architecture")
        .and_then(|architecture| architecture.get("input_modalities"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty());
    Some(ModelEntry {
        id: id.to_string(),
        input_modalities,
    })
}

fn read_limited(reader: impl Read, max_bytes: u64, endpoint: &str) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    reader
        .take(max_bytes + 1)
        .read_to_end(&mut body)
        .map_err(|error| format!("读取 {endpoint} 响应失败: {error}"))?;
    if body.len() as u64 > max_bytes {
        return Err(format!("{endpoint} 响应超过安全大小限制"));
    }
    Ok(body)
}

fn parse_json_probe(
    body: &[u8],
    selected_model: &str,
    request_id: Option<String>,
) -> Result<ResponsesProbeResult, String> {
    let payload: Value =
        serde_json::from_slice(body).map_err(|_| "/responses 未返回有效 JSON".to_string())?;
    validate_completed_response(&payload, selected_model)?;
    Ok(ResponsesProbeResult {
        protocol: "json".to_string(),
        model: selected_model.to_string(),
        request_id,
        completed: true,
    })
}

fn parse_sse_probe(
    reader: impl Read,
    selected_model: &str,
    request_id: Option<String>,
) -> Result<ResponsesProbeResult, String> {
    let mut reader = BufReader::new(reader.take(MAX_RESPONSES_PROBE_BYTES + 1));
    let mut line = String::new();
    let mut event_name = String::new();
    let mut event_data = String::new();
    let mut bytes_read = 0_u64;

    loop {
        line.clear();
        let count = reader
            .read_line(&mut line)
            .map_err(|error| format!("读取 /responses 流失败: {error}"))?;
        if count == 0 {
            if !event_data.is_empty()
                && process_sse_event(&event_name, &event_data, selected_model)?
            {
                return Ok(ResponsesProbeResult {
                    protocol: "sse".to_string(),
                    model: selected_model.to_string(),
                    request_id,
                    completed: true,
                });
            }
            break;
        }
        bytes_read += count as u64;
        if bytes_read > MAX_RESPONSES_PROBE_BYTES {
            return Err("/responses 流超过安全大小限制".to_string());
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            if process_sse_event(&event_name, &event_data, selected_model)? {
                return Ok(ResponsesProbeResult {
                    protocol: "sse".to_string(),
                    model: selected_model.to_string(),
                    request_id,
                    completed: true,
                });
            }
            event_name.clear();
            event_data.clear();
        } else if let Some(value) = trimmed.strip_prefix("event:") {
            event_name = value.trim().to_string();
        } else if let Some(value) = trimmed.strip_prefix("data:") {
            if !event_data.is_empty() {
                event_data.push('\n');
            }
            event_data.push_str(value.trim_start());
        }
    }

    Err("/responses 流在完成事件前断开".to_string())
}

fn process_sse_event(
    event_name: &str,
    event_data: &str,
    selected_model: &str,
) -> Result<bool, String> {
    if event_data.is_empty() || event_data == "[DONE]" {
        return Ok(false);
    }
    let payload: Value = serde_json::from_str(event_data)
        .map_err(|_| "/responses 流事件不是有效 JSON".to_string())?;
    let event_type = payload
        .get("type")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or(event_name);
    match event_type {
        "response.completed" => {
            let response = payload
                .get("response")
                .ok_or_else(|| "/responses 完成事件缺少 response".to_string())?;
            validate_completed_response(response, selected_model)?;
            Ok(true)
        }
        "response.failed" | "response.incomplete" | "error" => {
            Err("/responses 返回失败或未完成事件".to_string())
        }
        _ => Ok(false),
    }
}

fn validate_completed_response(payload: &Value, selected_model: &str) -> Result<(), String> {
    if payload.get("object").and_then(Value::as_str) != Some("response") {
        return Err("/responses 返回结构缺少 response object".to_string());
    }
    if payload.get("status").and_then(Value::as_str) != Some("completed") {
        return Err("/responses 未返回 completed 状态".to_string());
    }
    if payload
        .get("id")
        .and_then(Value::as_str)
        .is_none_or(|id| id.trim().is_empty())
    {
        return Err("/responses 返回结构缺少 response id".to_string());
    }
    let returned_model = payload
        .get("model")
        .and_then(Value::as_str)
        .ok_or_else(|| "/responses 返回结构缺少 model".to_string())?;
    if returned_model != selected_model {
        return Err(format!(
            "/responses 返回模型与选择不一致（选择 {selected_model}，返回 {returned_model}）"
        ));
    }
    if !payload.get("output").is_some_and(Value::is_array) {
        return Err("/responses 返回结构缺少 output 数组".to_string());
    }
    Ok(())
}

pub fn is_local_ollama_gateway(gateway: &str) -> bool {
    Url::parse(gateway)
        .ok()
        .map(|url| {
            matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"))
                && url.port_or_known_default() == Some(11434)
        })
        .unwrap_or(false)
}

pub fn is_ollama_gateway(gateway: &str) -> bool {
    Url::parse(gateway)
        .ok()
        .and_then(|url| url.port_or_known_default())
        == Some(11434)
}

fn local_ollama_connection_error() -> String {
    if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        return "未检测到 Windows 本机 Ollama 服务。当前是 Windows ARM64；请确认 Ollama 已安装并启动，或填写 macOS 宿主机可访问地址。127.0.0.1 只指向此 Windows VM".to_string();
    }
    "未检测到本机 Ollama 服务。请先安装并启动 Ollama；如果 Ollama 在虚拟机宿主机上，请填写宿主机可访问地址，不能使用 127.0.0.1".to_string()
}

fn remote_ollama_connection_error(gateway: &str) -> String {
    if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        return format!(
            "无法连接 Ollama：{gateway}。该地址的 11434 端口没有服务监听。若 Ollama 运行在 Parallels 的 macOS 宿主机，请先启动宿主机桥接，并使用 http://10.211.55.2:11434/v1；macOS Wi-Fi 地址不能替代未开放的 Ollama 监听地址"
        );
    }
    format!("无法连接 Ollama：{gateway}。请确认 Ollama 已启动，并监听当前设备可访问的网络接口")
}

fn http_error(operation: &str, gateway: &str, error: ureq::Error) -> String {
    match error {
        ureq::Error::Status(code, _) if code == 401 || code == 403 => {
            format!("{operation} 鉴权失败（HTTP {code}），请检查 Access Key")
        }
        ureq::Error::Status(code, _) => format!("{operation} 返回 HTTP {code}"),
        ureq::Error::Transport(_) if is_local_ollama_gateway(gateway) => {
            local_ollama_connection_error()
        }
        ureq::Error::Transport(_) if is_ollama_gateway(gateway) => {
            remote_ollama_connection_error(gateway)
        }
        ureq::Error::Transport(error) => format!("无法连接 Router：{error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Cursor, Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc::{self, Receiver},
        thread::{self, JoinHandle},
    };

    fn completed_response(model: &str) -> Value {
        json!({
            "id": "resp_probe",
            "object": "response",
            "status": "completed",
            "model": model,
            "output": [],
        })
    }

    fn spawn_router(responses: Vec<String>) -> (String, Receiver<String>, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test router");
        let address = listener.local_addr().expect("test router address");
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept test request");
                let request = read_request(&mut stream);
                sender.send(request).expect("capture request");
                stream
                    .write_all(response.as_bytes())
                    .expect("write response");
            }
        });
        (format!("http://{address}/v1"), receiver, handle)
    }

    fn read_request(stream: &mut TcpStream) -> String {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set test read timeout");
        let mut request = Vec::new();
        let mut buffer = [0_u8; 2048];
        loop {
            let count = stream.read(&mut buffer).expect("read request");
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            let Some(header_end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") else {
                continue;
            };
            let body_start = header_end + 4;
            let headers = String::from_utf8_lossy(&request[..header_end]);
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.split_once(':').and_then(|(name, value)| {
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    })
                })
                .unwrap_or(0);
            if request.len() >= body_start + content_length {
                break;
            }
        }
        String::from_utf8(request).expect("utf8 test request")
    }

    fn http_json(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn json_probe_validates_structure_without_retaining_output() {
        let mut payload = completed_response("model-a");
        payload["output"] = json!([{
            "type": "message",
            "content": [{ "type": "output_text", "text": "sensitive output" }]
        }]);
        let result = parse_json_probe(
            serde_json::to_vec(&payload).unwrap().as_slice(),
            "model-a",
            Some("request-1".to_string()),
        )
        .unwrap();
        assert_eq!(result.protocol, "json");
        assert_eq!(result.model, "model-a");
        assert_eq!(result.request_id.as_deref(), Some("request-1"));
        assert!(result.completed);
        assert!(!format!("{result:?}").contains("sensitive output"));
    }

    #[test]
    fn sse_probe_requires_completed_event() {
        let completed = json!({
            "type": "response.completed",
            "response": completed_response("model-a"),
            "sequence_number": 2,
        });
        let stream = format!(
            "event: response.created\ndata: {{\"type\":\"response.created\"}}\n\n\
             event: response.completed\ndata: {completed}\n\n"
        );
        let result = parse_sse_probe(
            Cursor::new(stream),
            "model-a",
            Some("request-2".to_string()),
        )
        .unwrap();
        assert_eq!(result.protocol, "sse");
        assert!(result.completed);
    }

    #[test]
    fn sse_probe_rejects_disconnect_without_leaking_delta() {
        let stream =
            "event: response.output_text.delta\ndata: {\"type\":\"response.output_text.delta\",\"delta\":\"private text\"}\n\n";
        let error = parse_sse_probe(Cursor::new(stream), "model-a", None).unwrap_err();
        assert!(error.contains("完成事件前断开"));
        assert!(!error.contains("private text"));
    }

    #[test]
    fn probe_rejects_model_mismatch() {
        let error = validate_completed_response(&completed_response("model-b"), "model-a")
            .expect_err("model mismatch");
        assert!(error.contains("模型与选择不一致"));
    }

    #[test]
    fn model_parser_deduplicates_and_rejects_empty_catalog() {
        let payload = json!({
            "data": [
                { "id": "model-b" },
                { "id": "model-a" },
                { "id": "model-a" },
                { "id": " " }
            ]
        });
        let body = serde_json::to_vec(&payload).unwrap();
        let models = parse_models_response(&body, "http://router.test/v1").unwrap();
        let ids = models.iter().map(|entry| entry.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["model-a", "model-b"]);
        assert!(models.iter().all(|entry| entry.input_modalities.is_none()));
        assert!(parse_models_response(b"{\"data\":[]}", "http://router.test/v1").is_err());
    }

    #[test]
    fn model_parser_extracts_input_modalities_when_declared() {
        let payload = json!({
            "data": [
                {
                    "id": "vision-model",
                    "architecture": { "input_modalities": ["Text", "IMAGE", " "] }
                },
                {
                    "id": "text-model",
                    "architecture": { "input_modalities": [] }
                }
            ]
        });
        let body = serde_json::to_vec(&payload).unwrap();
        let models = parse_models_response(&body, "http://router.test/v1").unwrap();
        assert_eq!(models[0].id, "text-model");
        assert!(models[0].input_modalities.is_none());
        assert_eq!(models[1].id, "vision-model");
        assert_eq!(
            models[1].input_modalities.as_deref(),
            Some(["text".to_string(), "image".to_string()].as_slice())
        );
    }

    #[test]
    fn probe_request_is_fixed_and_contains_no_customer_data() {
        let request = probe_request("model-a");
        assert_eq!(request["input"], "Return OK.");
        assert_eq!(request["model"], "model-a");
        assert_eq!(request["stream"], true);
        assert_eq!(request["max_output_tokens"], 16);
    }

    #[test]
    fn models_and_responses_share_auth_and_network_path() {
        let models_body = r#"{"object":"list","data":[{"id":"model-a"}]}"#;
        let completed = json!({
            "type": "response.completed",
            "response": completed_response("model-a"),
        });
        let sse_body = format!("event: response.completed\ndata: {completed}\n\n");
        let sse_response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nx-request-id: request-shared-path\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{sse_body}",
            sse_body.len()
        );
        let (gateway, requests, server) =
            spawn_router(vec![http_json("200 OK", models_body), sse_response]);
        let client = RouterClient::new(&gateway, Some("test-secret"));

        assert_eq!(client.fetch_models().unwrap(), vec!["model-a"]);
        let probe = client.probe_responses("model-a").unwrap();
        assert_eq!(probe.protocol, "sse");
        assert_eq!(probe.request_id.as_deref(), Some("request-shared-path"));

        let models_request = requests.recv().expect("models request");
        let responses_request = requests.recv().expect("responses request");
        server.join().expect("test router");
        assert!(models_request.starts_with("GET /v1/models "));
        assert!(responses_request.starts_with("POST /v1/responses "));
        for request in [&models_request, &responses_request] {
            assert!(request.contains("Authorization: Bearer test-secret"));
        }
        assert!(responses_request.contains(r#""input":"Return OK.""#));
        assert!(responses_request.contains(r#""stream":true"#));
    }

    #[test]
    fn models_success_does_not_hide_responses_failure() {
        let models_body = r#"{"object":"list","data":[{"id":"model-a"}]}"#;
        let error_body = r#"{"error":{"message":"unsupported"}}"#;
        let (gateway, _requests, server) = spawn_router(vec![
            http_json("200 OK", models_body),
            http_json("404 Not Found", error_body),
        ]);
        let client = RouterClient::new(&gateway, None);
        assert_eq!(client.fetch_models().unwrap(), vec!["model-a"]);
        let error = client.probe_responses("model-a").unwrap_err();
        server.join().expect("test router");
        assert_eq!(error, "POST /responses 返回 HTTP 404");
        assert!(!error.contains("unsupported"));
    }
}
