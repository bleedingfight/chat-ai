const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const marked = window.marked;

let messageInputEl;
let messageOutputEl;
let chatLogEl;
let apiKeyEl;
let modelSelectEl;
let apiSelectEl;
let apiUrlEl;

// API 配置
const API_CONFIGS = {
  deepseek: {
    url: "https://api.deepseek.com/v1/chat/completions",
    needsKey: true,
    models: ["deepseek-chat", "deepseek-coder", "mixtral-8x7b", "llama2-70b"],
  },
  openai: {
    url: "https://api.openai.com/v1/chat/completions",
    needsKey: true,
    models: ["gpt-4", "gpt-3.5-turbo"],
  },
  custom: {
    url: "https://api.moonshot.cn/v1/chat/completions",
    needsKey: true,
    needsUrl: true,
  },
};

// 配置 marked
marked.setOptions({
  highlight: function (code, lang) {
    return code;
  },
  breaks: true,
  gfm: true,
  // 添加数学公式处理
  extensions: [
    {
      name: "math",
      level: "inline",
      start(src) {
        return src.match(/\$/)?.index;
      },
      tokenizer(src, tokens) {
        const match = src.match(/^\$+([^$\n]+?)\$+/);
        if (match) {
          return {
            type: "math",
            raw: match[0],
            text: match[1].trim(),
          };
        }
      },
      renderer(token) {
        return token.raw;
      },
    },
    {
      name: "math",
      level: "block",
      start(src) {
        return src.match(/\$\$/)?.index;
      },
      tokenizer(src, tokens) {
        const match = src.match(/^\$\$([\s\S]+?)\$\$/);
        if (match) {
          return {
            type: "math",
            raw: match[0],
            text: match[1].trim(),
          };
        }
      },
      renderer(token) {
        return token.raw;
      },
    },
  ],
});

// 主题切换
function toggleTheme() {
  const isDark = document.body.getAttribute("data-theme") === "dark";
  document.body.setAttribute("data-theme", isDark ? "light" : "dark");
  localStorage.setItem("theme", isDark ? "light" : "dark");
}

// 初始化主题
function initTheme() {
  const savedTheme = localStorage.getItem("theme") || "light";
  document.body.setAttribute("data-theme", savedTheme);
}

// 更新模型选项
function updateModelOptions(apiType) {
  const config = API_CONFIGS[apiType];
  const models = config?.models || [];

  modelSelectEl.innerHTML = models
    .map((model) => `<option value="${model}">${model}</option>`)
    .join("");

  if (models.length > 0) {
    modelSelectEl.value = models[0];
  }
}

// 添加获取模型列表的函数
async function fetchAvailableModels(apiUrl, apiKey) {
  // 显示加载状态
  modelSelectEl.innerHTML = '<option value="">正在获取模型列表...</option>';

  try {
    const response = await invoke("fetch_models", {
      apiUrl,
      apiKey,
    });

    if (response.models && response.models.length > 0) {
      // 更新模型下拉列表
      modelSelectEl.innerHTML = response.models
        .map((model) => `<option value="${model}">${model}</option>`)
        .join("");

      modelSelectEl.value = response.models[0];
      saveSettings();

      // 清除之前的错误消息
      messageOutputEl.textContent = "";
    } else {
      modelSelectEl.innerHTML = '<option value="">未找到可用模型</option>';
    }
  } catch (error) {
    console.error("获取模型列表失败:", error);
    modelSelectEl.innerHTML = '<option value="">获取模型列表失败</option>';
    messageOutputEl.textContent = `错误：${error}`;
  }
}

// 防抖函数
function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

// 修改 handleApiChange 函数
async function handleApiChange() {
  const apiType = apiSelectEl.value;
  const config = API_CONFIGS[apiType];

  if (!config) {
    console.error("未知的 API 类型:", apiType);
    return;
  }

  // 显示/隐藏输入框
  apiUrlEl.style.display = config.needsUrl ? "block" : "none";
  apiKeyEl.style.display = config.needsKey ? "block" : "none";

  // 如果有预定义的 URL，设置它
  if (config.url) {
    apiUrlEl.value = config.url;
  }

  if (config.needsUrl) {
    // 对于自定义 API，尝试获取模型列表
    const apiUrl = apiUrlEl.value.trim();
    const apiKey = apiKeyEl.value.trim();
    if (apiUrl && apiKey) {
      try {
        await fetchAvailableModels(apiUrl, apiKey);
        saveSettings();
      } catch (error) {
        console.error("获取模型列表失败:", error);
        modelSelectEl.innerHTML = '<option value="">获取模型列表失败</option>';
      }
    } else {
      modelSelectEl.innerHTML = '<option value="">请先填写必要的配置</option>';
    }
  } else if (config.models) {
    // 使用预定义的模型列表
    updateModelOptions(apiType);
    saveSettings();
  }
}

// 保存和加载设置
async function saveSettings() {
  // 加密存储 API key 和 URL
  const apiKey = apiKeyEl.value.trim();
  const apiUrl = apiUrlEl.value.trim();

  if (apiKey) {
    try {
      await invoke("save_api_key", { apiKey });
    } catch (error) {
      console.error("保存 API key 失败:", error);
    }
  }

  if (apiUrl) {
    try {
      await invoke("save_api_url", { apiUrl });
    } catch (error) {
      console.error("保存 API URL 失败:", error);
    }
  }

  // 只在 localStorage 中保存非敏感信息
  const settings = {
    apiType: apiSelectEl.value,
    model: modelSelectEl.value,
  };
  localStorage.setItem("chat-settings", JSON.stringify(settings));
}

// 修改 loadSettings 函数
async function loadSettings() {
  const settings = JSON.parse(localStorage.getItem("chat-settings") || "{}");

  // 从加密存储获取 API key 和 URL
  try {
    const [savedApiKey, savedApiUrl] = await Promise.all([
      invoke("get_api_key", {
        api_keys_path: await invoke("get_cache_directory").then(
          (dir) => `${dir}/api_keys.enc`,
        ),
      }).catch(() => null),
      invoke("get_api_url").catch(() => null),
    ]);

    if (savedApiKey) {
      apiKeyEl.value = savedApiKey;
    }

    if (savedApiUrl) {
      apiUrlEl.value = savedApiUrl;
    }
  } catch (error) {
    console.error("获取加密配置失败:", error);
  }

  // 设置 API 类型
  if (settings.apiType) {
    apiSelectEl.value = settings.apiType;
  }

  // 处理 API 类型相关的设置
  const apiType = settings.apiType || "deepseek";
  const config = API_CONFIGS[apiType];

  if (config) {
    // 设置显示/隐藏状态
    apiUrlEl.style.display = config.needsUrl ? "block" : "none";
    apiKeyEl.style.display = config.needsKey ? "block" : "none";

    // 如果有预定义的 URL，设置它
    if (config.url) {
      apiUrlEl.value = config.url;
    }

    if (config.needsUrl) {
      // 如果需要自定义 URL 且有完整的 API 信息，自动获取模型列表
      const apiUrl = apiUrlEl.value.trim();
      const apiKey = apiKeyEl.value.trim();
      if (apiUrl && apiKey) {
        try {
          await fetchAvailableModels(apiUrl, apiKey);
        } catch (error) {
          console.error("自动获取模型列表失败:", error);
          modelSelectEl.innerHTML =
            '<option value="">获取模型列表失败</option>';
        }
      } else {
        modelSelectEl.innerHTML =
          '<option value="">请先填写必要的配置</option>';
      }
    } else if (config.models) {
      // 使用预定义的模型列表
      updateModelOptions(apiType);
    }
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
let currentStreamDiv = null;
let currentStreamContent = "";
let userScrolled = false;

// 监听聊天记录的滚动事件
function setupScrollListener() {
  chatLogEl.addEventListener("scroll", () => {
    // 检查是否是用户手动滚动
    const isAtBottom =
      chatLogEl.scrollHeight - chatLogEl.scrollTop <=
      chatLogEl.clientHeight + 50;
    userScrolled = !isAtBottom;
  });
}

// 智能滚动函数
function smartScroll() {
  if (!userScrolled) {
    chatLogEl.scrollTop = chatLogEl.scrollHeight;
  }
}

// 设置流式响应监听器
async function setupStreamListener() {
  await listen("stream-response", (event) => {
    if (currentStreamDiv) {
      // 第一次收到响应时
      if (!currentStreamContent) {
        // 移除思考状态div（它是currentStreamDiv的前一个兄弟元素）
        const thinkingDiv = currentStreamDiv.previousSibling;
        if (thinkingDiv) {
          thinkingDiv.remove();
        }
        // 显示响应div
        currentStreamDiv.style.display = "block";
      }

      currentStreamContent += event.payload;
      currentStreamDiv.innerHTML = `AI：${marked.parse(currentStreamContent)}`;

      // 触发 MathJax 重新渲染
      if (window.MathJax) {
        window.MathJax.typesetPromise([currentStreamDiv]).catch((err) => {
          console.error("MathJax rendering failed:", err);
        });
      }

      smartScroll();
    }
  });
}

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
    const config = API_CONFIGS[apiType];

    let apiUrl;
    if (config.needsUrl) {
      apiUrl = apiUrlEl.value.trim();
      if (!apiUrl) {
        messageOutputEl.textContent = "请输入自定义 API 地址！";
        return;
      }
    } else {
      apiUrl = config.url;
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
      contextMessage = `${contextPairs.join("\n")}\n${message}`;
    }

    // 添加用户消息到历史记录
    conversationHistory.push({
      role: "user",
      content: message,
    });

    appendMessage("user", message);

    // 创建新的流式响应div
    currentStreamContent = "";
    // 创建思考状态的div
    const thinkingDiv = document.createElement("div");
    thinkingDiv.className = "message ai";
    const thinkingSpan = document.createElement("span");
    thinkingSpan.className = "thinking-text";
    thinkingSpan.textContent = "AI：正在思考";
    thinkingDiv.appendChild(thinkingSpan);
    chatLogEl.appendChild(thinkingDiv);

    // 创建用于显示响应的div
    currentStreamDiv = document.createElement("div");
    currentStreamDiv.className = "message ai";
    currentStreamDiv.style.display = "none";
    chatLogEl.appendChild(currentStreamDiv);

    try {
      const response = await invoke("chat", {
        message: contextMessage,
        apiKey,
        apiUrl,
        model,
        history: [],
      });

      // 流式响应完成后，保存到历史记录
      conversationHistory.push({
        role: "assistant",
        content: currentStreamContent,
      });

      // 重置滚动状态，为下一次对话准备
      userScrolled = false;
    } catch (error) {
      currentStreamDiv.remove();
      messageOutputEl.textContent = `错误：${error}`;
      conversationHistory.pop(); // 发生错误时，回滚最后一条用户消息
    }
  } catch (error) {
    console.error("Error:", error);
    messageOutputEl.textContent = `错误：${error}`;
  }
}

// 修改 appendMessage 函数，确保消息按顺序显示
function appendMessage(role, content) {
  const messageDiv = document.createElement("div");
  messageDiv.className = `message ${role}`;

  if (role === "user") {
    messageDiv.textContent = `你：${content}`;
  } else {
    // 对 AI 回复使用 Markdown 渲染
    messageDiv.innerHTML = `AI：${marked.parse(content)}`;

    // 触发 MathJax 重新渲染
    if (window.MathJax) {
      window.MathJax.typesetPromise([messageDiv]).catch((err) => {
        console.error("MathJax rendering failed:", err);
      });
    }
  }

  chatLogEl.appendChild(messageDiv);
  smartScroll();
}

// 添加清除历史的函数
function clearHistory() {
  conversationHistory = [];
  chatLogEl.innerHTML = "";
  messageOutputEl.textContent = "";
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

  // 设置流式响应监听器
  await setupStreamListener();

  // 设置滚动监听器
  setupScrollListener();

  // 加载设置（现在是异步的）
  await loadSettings();

  // 设置主题切换按钮事件
  document
    .querySelector("#theme-toggle")
    .addEventListener("click", toggleTheme);

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

  // 统一处理 API 配置变化
  const handleApiConfigChange = debounce(async () => {
    const apiUrl = apiUrlEl.value.trim();
    const apiKey = apiKeyEl.value.trim();

    // 如果是自定义 API 或者配置发生变化，尝试获取模型列表
    if (apiUrl && apiKey) {
      try {
        await fetchAvailableModels(apiUrl, apiKey);
        // 如果成功获取模型列表，保存配置
        saveSettings();
      } catch (error) {
        console.error("获取模型列表失败:", error);
        modelSelectEl.innerHTML = '<option value="">获取模型列表失败</option>';
      }
    } else {
      modelSelectEl.innerHTML =
        '<option value="">请输入完整的 API 配置</option>';
    }
  }, 500); // 500ms 的防抖延迟

  // 为 API 配置相关输入框添加事件监听
  apiUrlEl.addEventListener("input", handleApiConfigChange);
  apiKeyEl.addEventListener("input", handleApiConfigChange);

  // 初始触发一次检查
  if (apiSelectEl.value === "custom") {
    handleApiConfigChange();
  }

  // 添加清除历史按钮的事件监听
  document
    .querySelector("#clear-history")
    ?.addEventListener("click", clearHistory);
});
