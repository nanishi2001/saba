use alloc::{string::{String, ToString}, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
// URLを示す構造体
pub struct Url {
	url:String,
	host:String,
	port:String,
	path:String,
	searchpart:String,
}

// 構造体URLのメソッド群
impl Url {
	// 構造体宣言時に実行されるコンストラクタ
	pub fn new(url:String) -> Self {
		Self {
			url,
			host: "".to_string(),
			port: "".to_string(),
			path: "".to_string(),
			searchpart: "".to_string(),
		}
	}
	// URLをパースするメソッド
	pub fn parse(&mut self) -> Result<Self, String> {
		// HTTP以外のスキーマの場合(今回の実装ではHTTPのみがスコープ)
		if !self.is_http() {
			return Err("Only HTTP scheme is supported.".to_string());
		}

		// url以外のプロパティを抽出
		self.host = self.extract_host();
		self.port = self.extract_port();
		self.path = self.extract_path();
		self.searchpart = self.extract_searchpart();

		// Result型に即した返値を返す
		return Ok(self.clone());
	}

	// URLのスキーマがHTTPかどうかを判定(今回の実装ではHTTPのみがスコープ)
	fn is_http(&mut self) -> bool {
		if self.url.contains("http://") {
			return true;
		}
		return false;	// Rustの機能で明示的にreturnを書かなくても関数の最終行が返値になるが書いたほうがわかりやすいので記載
	}

	// URLからhostを取得するメソッド
	fn extract_host(&self) -> String {
		// 先頭からhttp://までを取り除き、2つめの / まで分割した配列を作成
		let url_parts: Vec<&str> = self.url.trim_start_matches("http://").splitn(2, "/").collect();
		// 配列の先頭(host)に : (port番号)が含まれているか、含まれていたら何番目かを探す
		if let Some(index) = url_parts[0].find(':') {
			return url_parts[0][..index].to_string();	// 含まれている場合はport番号までを返す
		} else {
			return url_parts[0].to_string();	// 含まれていない場合はそのまま返す
		}
	}
	// port番号を取得するメソッド
	fn extract_port(&self) -> String {
		let url_patrs: Vec<&str> = self.url.trim_start_matches("http://").splitn(2, "/").collect();
		// port番号が含まれているかを判定
		if let Some(index) = url_patrs[0].find(':') {
			return url_patrs[0][index + 1..].to_string();	// 含まれている場合indexの次文字から末尾までがport番号
		} else {
			return "80".to_string();	// 含まれていない場合は80を返す
		}
	}
	// pathを取得するメソッド
	fn extract_path(&self) -> String {
		let url_parts: Vec<&str> = self.url.trim_start_matches("http://").splitn(2, "/").collect();
		if url_parts.len() < 2 {
			return "".to_string();	// pathが存在しない場合
		}
		// pathが存在する場合
		let path_and_searchpart: Vec<&str> = url_parts[1].splitn(2, "?").collect();	// pathと?以降のクエリパラメータを分割
		return path_and_searchpart[0].to_string();	// pathを返す
	}
	// クエリパラメータ(searchpart)を取得するメソッド
	fn extract_searchpart(&self) -> String {
		let url_parts: Vec<&str> = self.url.trim_start_matches("http://").splitn(2, "/").collect();
		if  url_parts.len() < 2 {
			return "".to_string();
		}
		let path_and_serchpart: Vec<&str> = url_parts[1].splitn(2, "?").collect();
		if path_and_serchpart.len() < 2 {
			return "".to_string();	// pathがあってもクエリパラメータがない場合
		} else {
			return path_and_serchpart[1].to_string();
		}
	}

	// ゲッターメソッド
	pub fn host(&self) -> String {
		return self.host.clone();
	}
	pub fn port(&self) -> String {
		return self.port.clone();
	}
	pub fn path(&self) -> String {
		return self.path.clone();
	}
	pub fn searchpart(&self) -> String {
		return self.searchpart.clone();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// 成功時のテスト
	#[test]
	fn test_url_host() {
		let url = "http://example.com".to_string();
		let expected = Ok(Url {
			url: url.clone(),
			host: "example.com".to_string(),
			port: "80".to_string(),
			path: "".to_string(),
			searchpart: "".to_string(),
		});
		assert_eq!(expected, Url::new(url).parse());
	}

	#[test]
	fn test_url_host_port() {
		let url="http://example.com:8888".to_string();
		let expected = Ok(Url {
			url: url.clone(),
			host: "example.com".to_string(),
			port: "8888".to_string(),
			path: "".to_string(),
			searchpart: "".to_string(),
		});
		assert_eq!(expected, Url::new(url).parse());
	}

	#[test]
	fn test_url_host_port_path() {
		let url = "http://example.com:8888/index.html".to_string();
		let expected = Ok(Url {
			url: url.clone(),
			host: "example.com".to_string(),
			port: "8888".to_string(),
			path: "index.html".to_string(),
			searchpart: "".to_string(),
		});
		assert_eq!(expected, Url::new(url).parse());
	}

	#[test]
	fn test_url_host_path_searchpart() {
		let url = "http://example.com/index.html".to_string();
		let expected = Ok(Url {
			url: url.clone(),
			host: "example.com".to_string(),
			port: "80".to_string(),
			path: "index.html".to_string(),
			searchpart: "".to_string(),
		});
		assert_eq!(expected, Url::new(url).parse());
	}

	#[test]
	fn test_url_host_port_path_searchpart() {
		let url = "http://example.com:8888/index.html?a=123&b=456".to_string();
		let expected = Ok(Url {
			url: url.clone(),
			host: "example.com".to_string(),
			port: "8888".to_string(),
			path: "index.html".to_string(),
			searchpart: "a=123&b=456".to_string(),
		});
		assert_eq!(expected, Url::new(url).parse());
	}

	// エラー時のテスト
	#[test]
	fn test_no_scheme() {
		let url = "example.com".to_string();
		let expected = Err("Only HTTP scheme is supported.".to_string());
		assert_eq!(expected, Url::new(url).parse());
	}

	#[test]
	fn test_unsupported_scheme() {
		let url = "https://example.com:8888/index.html?a=123&b=456".to_string();
		let expected = Err("Only HTTP scheme is supported.".to_string());
		assert_eq!(expected, Url::new(url).parse());
	}
}
