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
  gfm: true
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

// 处理 API 选择变化
function handleApiChange() {
  const apiType = apiSelectEl.value;
  
  // 显示/隐藏自定义 API 输入框
  apiUrlEl.classList.toggle('show', apiType === 'custom');
  
  // 更新模型选项
  if (apiType !== 'custom') {
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

function loadSettings() {
  const settings = JSON.parse(localStorage.getItem('chat-settings') || '{}');
  
  if (settings.apiKey) {
    apiKeyEl.value = settings.apiKey;
  }
  if (settings.apiType) {
    apiSelectEl.value = settings.apiType;
    handleApiChange();
  }
  if (settings.apiUrl) {
    apiUrlEl.value = settings.apiUrl;
  }
  if (settings.model) {
    modelSelectEl.value = settings.model;
  }
}

function appendMessage(role, content) {
  const messageDiv = document.createElement('div');
  messageDiv.className = `message ${role}`;
  
  if (role === 'user') {
    messageDiv.textContent = `你：${content}`;
  } else {
    // 对 AI 回复使用 Markdown 渲染
    messageDiv.innerHTML = `AI：${marked.parse(content)}`;
  }
  
  chatLogEl.appendChild(messageDiv);
  chatLogEl.scrollTop = chatLogEl.scrollHeight;
}

async function chat() {
  try {
    // 获取用户输入的消息
    const message = messageInputEl.value.trim();
    if (!message) {
      messageOutputEl.textContent = "请输入你的问题！";
      return;
    }

    // 检查 API Key
    const apiKey = apiKeyEl.value.trim();
    if (!apiKey) {
      messageOutputEl.textContent = "请输入 API Key！";
      return;
    }

    // 获取 API URL
    const apiType = apiSelectEl.value;
    let apiUrl = API_ENDPOINTS[apiType];
    if (apiType === 'custom') {
      apiUrl = apiUrlEl.value.trim();
      if (!apiUrl) {
        messageOutputEl.textContent = "请输入自定义 API 地址！";
        return;
      }
    }

    // 获取选择的模型
    const model = modelSelectEl.value;

    // 清空错误信息
    messageOutputEl.textContent = "";
    
    // 禁用输入框和按钮，防止重复发送
    messageInputEl.disabled = true;
    document.querySelector('button[type="submit"]').disabled = true;

    // 显示用户消息
    appendMessage('user', message);

    // 显示正在思考的提示
    const thinkingDiv = document.createElement('div');
    thinkingDiv.className = 'message ai thinking';
    thinkingDiv.textContent = 'AI：正在思考...';
    chatLogEl.appendChild(thinkingDiv);

    // 调用后端 chat 函数
    const response = await invoke("chat", { 
      message,
      apiKey,
      apiUrl,
      model
    });
    
    // 移除思考提示并显示回复
    chatLogEl.removeChild(thinkingDiv);
    appendMessage('ai', response);

    // 清空输入框
    messageInputEl.value = "";
  } catch (error) {
    console.error("Error:", error);
    messageOutputEl.textContent = `错误：${error}`;
    
    // 如果有思考提示，移除它
    const thinkingDiv = chatLogEl.querySelector('.thinking');
    if (thinkingDiv) {
      chatLogEl.removeChild(thinkingDiv);
    }
  } finally {
    // 重新启用输入框和按钮
    messageInputEl.disabled = false;
    document.querySelector('button[type="submit"]').disabled = false;
  }
}

window.addEventListener("DOMContentLoaded", () => {
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
  
  // 加载设置
  loadSettings();

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
});
