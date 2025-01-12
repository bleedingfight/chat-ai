const { invoke } = window.__TAURI__.core;
const marked = window.marked;

let messageInputEl;
let messageOutputEl;
let chatLogEl;
let apiKeyEl;
let modelSelectEl;
let apiSelectEl;
let apiUrlEl;

// API 端点配置
const API_ENDPOINTS = {
  deepseek: "https://api.deepseek.com/chat/completions",
  openai: "https://api.openai.com/v1/chat/completions",
};

// 模型配置
const API_MODELS = {
  deepseek: ["deepseek-chat", "deepseek-coder", "mixtral-8x7b", "llama2-70b"],
  openai: ["gpt-4", "gpt-3.5-turbo"],
};

// 配置 marked
marked.setOptions({
  highlight: function(code, lang) {
    return code;
  },
  breaks: true,
  gfm: true,
  // 添加数学公式处理
  extensions: [{
    name: 'math',
    level: 'inline',
    start(src) { return src.match(/\$/)?.index; },
    tokenizer(src, tokens) {
      const match = src.match(/^\$+([^$\n]+?)\$+/);
      if (match) {
        return {
          type: 'math',
          raw: match[0],
          text: match[1].trim()
        };
      }
    },
    renderer(token) {
      return token.raw;
    }
  }, {
    name: 'math',
    level: 'block',
    start(src) { return src.match(/\$\$/)?.index; },
    tokenizer(src, tokens) {
      const match = src.match(/^\$\$([\s\S]+?)\$\$/);
      if (match) {
        return {
          type: 'math',
          raw: match[0],
          text: match[1].trim()
        };
      }
    },
    renderer(token) {
      return token.raw;
    }
  }]
});

// 主题切换
function toggleTheme() {
  const isDark = document.body.getAttribute('data-theme') === 'dark';
  document.body.setAttribute('data-theme', isDark ? 'light' : 'dark');
  localStorage.setItem('theme', isDark ? 'light' : 'dark');
}

// 初始化主题
function initTheme() {
  const savedTheme = localStorage.getItem('theme') || 'light';
  document.body.setAttribute('data-theme', savedTheme);
}

// 更新模型选项
function updateModelOptions(apiType) {
  const models = API_MODELS[apiType] || [];
  modelSelectEl.innerHTML = models.map(model => 
    `<option value="${model}">${model}</option>`
  ).join('');
  
  if (models.length > 0) {
    modelSelectEl.value = models[0];
  }
}

// 添加获取模型列表的函数
async function fetchAvailableModels(apiUrl, apiKey) {
  try {
    const response = await invoke("fetch_models", {
      apiUrl,
      apiKey
    });
    
    // 更新模型下拉列表
    modelSelectEl.innerHTML = response.models.map(model => 
      `<option value="${model}">${model}</option>`
    ).join('');
    
    if (response.models.length > 0) {
      modelSelectEl.value = response.models[0];
      saveSettings();
    }
  } catch (error) {
    console.error("获取模型列表失败:", error);
    messageOutputEl.textContent = `获取模型列表失败：${error}`;
    // 清空模型列表
    modelSelectEl.innerHTML = '<option value="">无可用模型</option>';
  }
}

// 修改 handleApiChange 函数
function handleApiChange() {
  const apiType = apiSelectEl.value;
  
  // 显示/隐藏自定义 API 输入框
  apiUrlEl.classList.toggle('show', apiType === 'custom');
  
  if (apiType === 'custom') {
    // 如果已经有 URL 和 API Key，尝试获取模型列表
    const apiUrl = apiUrlEl.value.trim();
    const apiKey = apiKeyEl.value.trim();
    if (apiUrl && apiKey) {
      fetchAvailableModels(apiUrl, apiKey);
    } else {
      modelSelectEl.innerHTML = '<option value="">请先填写 API URL 和 Key</option>';
    }
  } else {
    // 使用预定义的模型列表
    updateModelOptions(apiType);
  }
}

// 保存和加载设置
function saveSettings() {
  const settings = {
    apiKey: apiKeyEl.value,
    apiType: apiSelectEl.value,
    apiUrl: apiUrlEl.value,
    model: modelSelectEl.value
  };
  localStorage.setItem('chat-settings', JSON.stringify(settings));
}

// 修改 loadSettings 函数
async function loadSettings() {
  const settings = JSON.parse(localStorage.getItem('chat-settings') || '{}');
  
  // 先加载基本设置
  if (settings.apiKey) {
    apiKeyEl.value = settings.apiKey;
  }
  if (settings.apiType) {
    apiSelectEl.value = settings.apiType;
  }
  if (settings.apiUrl) {
    apiUrlEl.value = settings.apiUrl;
  }

  // 处理 API 类型相关的设置
  if (settings.apiType === 'custom') {
    apiUrlEl.classList.add('show');
    // 如果是自定义模式且有完整的 API 信息，自动获取模型列表
    const apiUrl = settings.apiUrl?.trim();
    const apiKey = settings.apiKey?.trim();
    if (apiUrl && apiKey) {
      try {
        await fetchAvailableModels(apiUrl, apiKey);
      } catch (error) {
        console.error("自动获取模型列表失败:", error);
        modelSelectEl.innerHTML = '<option value="">获取模型列表失败</option>';
      }
    } else {
      modelSelectEl.innerHTML = '<option value="">请先填写 API URL 和 Key</option>';
    }
  } else {
    // 使用预定义的模型列表
    updateModelOptions(settings.apiType || 'deepseek');
  }

  // 最后设置选中的模型
  if (settings.model) {
    modelSelectEl.value = settings.model;
  }
}

// 添加一个 Map 来跟踪每个消息的状态
const pendingMessages = new Map();

// 在文件开头添加对话历史数组
let conversationHistory = [];

// 修改 chat 函数
async function chat() {
  try {
    const message = messageInputEl.value.trim();
    if (!message) {
      messageOutputEl.textContent = "请输入你的问题！";
      return;
    }

    const apiKey = apiKeyEl.value.trim();
    if (!apiKey) {
      messageOutputEl.textContent = "请输入 API Key！";
      return;
    }

    const apiType = apiSelectEl.value;
    let apiUrl = API_ENDPOINTS[apiType];
    if (apiType === 'custom') {
      apiUrl = apiUrlEl.value.trim();
      if (!apiUrl) {
        messageOutputEl.textContent = "请输入自定义 API 地址！";
        return;
      }
    }

    const model = modelSelectEl.value;
    messageOutputEl.textContent = "";
    messageInputEl.value = "";
    
    const messageId = Date.now().toString();
    
    // 构建带有上下文的消息
    let contextMessage = message;
    if (conversationHistory.length > 0) {
      const contextPairs = [];
      for (let i = 0; i < conversationHistory.length; i += 2) {
        const userMsg = conversationHistory[i];
        const aiMsg = conversationHistory[i + 1];
        if (userMsg && aiMsg) {
          contextPairs.push(`${userMsg.content}\n回答:${aiMsg.content}`);
        }
      }
      contextMessage = `${contextPairs.join('\n')}\n${message}`;
    }
    
    // 添加用户消息到历史记录
    conversationHistory.push({
      role: "user",
      content: message  // 保存原始消息到历史记录
    });
    
    appendMessage('user', message);
    
    const thinkingDiv = document.createElement('div');
    thinkingDiv.className = 'message ai thinking';
    thinkingDiv.textContent = 'AI：正在思考...';
    chatLogEl.appendChild(thinkingDiv);
    
    pendingMessages.set(messageId, {
      message: contextMessage,  // 使用带上下文的消息
      thinkingDiv,
      timestamp: Date.now()
    });

    // 添加日志输出
    console.log('发送请求数据:', {
      message: contextMessage,
      apiKey: apiKey.substring(0, 4) + '****', // 只显示 API Key 的前四位
      apiUrl,
      model,
      history: []
    });

    try {
      // 调用后端 chat 函数，使用带上下文的消息
      const response = await invoke("chat", { 
        message: contextMessage,  // 发送带上下文的消息
        apiKey,
        apiUrl,
        model,
        history: []  // 不使用 history 参数，因为上下文已经包含在 message 中
      });
      
      if (pendingMessages.has(messageId)) {
        thinkingDiv.remove();
        appendMessage('ai', response);
        
        // 添加 AI 回复到历史记录
        conversationHistory.push({
          role: "assistant",
          content: response
        });
        
        pendingMessages.delete(messageId);
      }
    } catch (error) {
      if (pendingMessages.has(messageId)) {
        thinkingDiv.remove();
        messageOutputEl.textContent = `错误：${error}`;
        pendingMessages.delete(messageId);
        // 发生错误时，回滚最后一条用户消息
        conversationHistory.pop();
      }
    }

  } catch (error) {
    console.error("Error:", error);
    messageOutputEl.textContent = `错误：${error}`;
  }
}

// 修改 appendMessage 函数，确保消息按顺序显示
function appendMessage(role, content) {
  const messageDiv = document.createElement('div');
  messageDiv.className = `message ${role}`;
  
  if (role === 'user') {
    messageDiv.textContent = `你：${content}`;
  } else {
    // 对 AI 回复使用 Markdown 渲染
    messageDiv.innerHTML = `AI：${marked.parse(content)}`;
    
    // 触发 MathJax 重新渲染
    if (window.MathJax) {
      window.MathJax.typesetPromise([messageDiv]).catch((err) => {
        console.error('MathJax rendering failed:', err);
      });
    }
  }
  
  chatLogEl.appendChild(messageDiv);
  chatLogEl.scrollTop = chatLogEl.scrollHeight;
}

// 添加清除历史的函数
function clearHistory() {
    conversationHistory = [];
    chatLogEl.innerHTML = '';
    messageOutputEl.textContent = '';
}

window.addEventListener("DOMContentLoaded", async () => {
  messageInputEl = document.querySelector("#greet-input");
  messageOutputEl = document.querySelector("#greet-msg");
  chatLogEl = document.querySelector("#chat-log");
  apiKeyEl = document.querySelector("#api-key");
  modelSelectEl = document.querySelector("#model-select");
  apiSelectEl = document.querySelector("#api-select");
  apiUrlEl = document.querySelector("#api-url");
  
  // 确保聊天记录区域存在
  if (!chatLogEl) {
    console.error("Chat log element not found!");
    return;
  }

  // 初始化主题
  initTheme();
  
  // 加载设置（现在是异步的）
  await loadSettings();

  // 设置主题切换按钮事件
  document.querySelector("#theme-toggle").addEventListener("click", toggleTheme);
  
  // 设置 API 选择变化事件
  apiSelectEl.addEventListener("change", () => {
    handleApiChange();
    saveSettings();
  });
  
  // 设置保存事件
  apiKeyEl.addEventListener("change", saveSettings);
  apiUrlEl.addEventListener("change", saveSettings);
  modelSelectEl.addEventListener("change", saveSettings);
  
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    chat();
  });

  // 添加输入框的回车事件监听
  messageInputEl.addEventListener("keypress", (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      chat();
    }
  });

  // 为自定义 API 输入框添加事件监听
  apiUrlEl.addEventListener("change", () => {
    if (apiSelectEl.value === 'custom') {
      const apiUrl = apiUrlEl.value.trim();
      const apiKey = apiKeyEl.value.trim();
      if (apiUrl && apiKey) {
        fetchAvailableModels(apiUrl, apiKey);
      }
    }
    saveSettings();
  });

  apiKeyEl.addEventListener("change", () => {
    if (apiSelectEl.value === 'custom') {
      const apiUrl = apiUrlEl.value.trim();
      const apiKey = apiKeyEl.value.trim();
      if (apiUrl && apiKey) {
        fetchAvailableModels(apiUrl, apiKey);
      }
    }
    saveSettings();
  });

  // 添加清除历史按钮的事件监听
  document.querySelector("#clear-history")?.addEventListener("click", clearHistory);
});
