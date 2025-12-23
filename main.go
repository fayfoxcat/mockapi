package main

import (
	"embed"
	"encoding/json"
	"fmt"
	"io"
	"io/fs"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"
)

//go:embed static/*
var staticFS embed.FS

const (
	AppName    = "mock-api-server"
	AppVersion = "1.0.0"
)

// MockAPI 模拟接口定义
type MockAPI struct {
	ID           string            `json:"id"`
	Name         string            `json:"name"`
	Method       string            `json:"method"`
	URL          string            `json:"url"`
	Headers      map[string]string `json:"headers"`
	ResponseBody string            `json:"responseBody"`
	Logs         []LogEntry        `json:"logs"`
	CreatedAt    string            `json:"createdAt"`
	UpdatedAt    string            `json:"updatedAt"`
}

// LogEntry 日志条目
type LogEntry struct {
	Timestamp   string            `json:"timestamp"`
	Method      string            `json:"method"`
	URL         string            `json:"url"`
	Headers     map[string]string `json:"headers"`
	RequestBody string            `json:"requestBody"`
	StatusCode  int               `json:"statusCode"`
	Error       string            `json:"error,omitempty"`
}

var (
	apis       []MockAPI
	apiMutex   sync.RWMutex
	baseDir    string // 可执行文件所在目录
	dataDir    string
	dataFile   string
	logDir     string
	pidFile    string
	appLogger  *log.Logger
	reqLogger  *log.Logger
	serverPort = "8344" // 默认端口
)

func main() {
	// 初始化基础目录（可执行文件所在目录）
	initBaseDir()

	// 解析所有参数，提取端口和命令
	var cmd string
	for i := 1; i < len(os.Args); i++ {
		arg := os.Args[i]
		if arg == "-p" && i+1 < len(os.Args) {
			serverPort = os.Args[i+1]
			i++ // 跳过端口值
		} else if !strings.HasPrefix(arg, "-") && cmd == "" {
			cmd = arg
		}
	}

	// 环境变量作为后备
	if serverPort == "8344" {
		if p := os.Getenv("PORT"); p != "" {
			serverPort = p
		}
	}

	// 无命令则前台运行
	if cmd == "" {
		runServer()
		return
	}

	switch cmd {
	case "start":
		startDaemon()
	case "stop":
		stopDaemon()
	case "restart":
		stopDaemon()
		time.Sleep(500 * time.Millisecond)
		startDaemon()
	case "status":
		showStatus()
	case "reset":
		resetData()
	case "help":
		showHelp()
	case "version":
		fmt.Printf("%s version %s\n", AppName, AppVersion)
	default:
		fmt.Printf("未知命令: %s\n", cmd)
		showHelp()
		os.Exit(1)
	}
}

// 初始化基础目录为可执行文件所在目录
func initBaseDir() {
	executable, err := os.Executable()
	if err != nil {
		baseDir = "."
	} else {
		baseDir = filepath.Dir(executable)
	}
	// 处理符号链接
	if realPath, err := filepath.EvalSymlinks(executable); err == nil {
		baseDir = filepath.Dir(realPath)
	}

	dataDir = filepath.Join(baseDir, "data")
	dataFile = filepath.Join(dataDir, "mock_apis.json")
	logDir = filepath.Join(baseDir, "logs")
	pidFile = filepath.Join(baseDir, "mock-api-server.pid")
}

func showHelp() {
	fmt.Printf(`%s v%s - Mock API 管理平台

用法:
  %s [选项] [命令]

命令:
  start     后台启动服务
  stop      停止服务
  restart   重启服务
  status    查看服务状态
  reset     重置数据(清空所有API配置)
  version   显示版本信息
  help      显示帮助信息

选项:
  -p <port> 指定服务端口(默认: 8344)

环境变量:
  PORT      服务端口(优先级低于 -p 参数)

示例:
  ./%s                  # 前台启动(默认端口)
  ./%s -p 9000          # 前台启动(指定端口)
  ./%s start            # 后台启动
  ./%s -p 9000 start    # 后台启动(指定端口)
  ./%s start -p 9000    # 后台启动(指定端口)
  ./%s stop             # 停止服务

数据目录: 可执行文件所在目录下的 data/ 和 logs/
`, AppName, AppVersion, AppName, AppName, AppName, AppName, AppName, AppName, AppName)
}

func getPid() int {
	data, err := os.ReadFile(pidFile)
	if err != nil {
		return 0
	}
	pid, err := strconv.Atoi(strings.TrimSpace(string(data)))
	if err != nil {
		return 0
	}
	return pid
}

func savePid(pid int) {
	os.WriteFile(pidFile, []byte(strconv.Itoa(pid)), 0644)
}

func removePid() {
	os.Remove(pidFile)
}

func isProcessRunning(pid int) bool {
	if pid <= 0 {
		return false
	}
	process, err := os.FindProcess(pid)
	if err != nil {
		return false
	}
	// 尝试发送信号检查进程
	err = process.Signal(os.Signal(nil))
	return err == nil
}

func startDaemon() {
	pid := getPid()
	if pid > 0 && isProcessRunning(pid) {
		fmt.Printf("服务已在运行中 (PID: %d)\n", pid)
		return
	}

	// 获取当前可执行文件路径
	executable, err := os.Executable()
	if err != nil {
		fmt.Printf("获取可执行文件路径失败: %v\n", err)
		os.Exit(1)
	}

	// 启动后台进程，传递端口参数
	cmd := exec.Command(executable, "-p", serverPort)
	cmd.Stdout = nil
	cmd.Stderr = nil
	cmd.Stdin = nil

	if err := cmd.Start(); err != nil {
		fmt.Printf("启动失败: %v\n", err)
		os.Exit(1)
	}

	savePid(cmd.Process.Pid)
	fmt.Printf("服务已启动 (PID: %d)\n", cmd.Process.Pid)
	fmt.Printf("访问地址: http://localhost:%s\n", serverPort)
}

func stopDaemon() {
	pid := getPid()
	if pid <= 0 {
		fmt.Println("服务未运行")
		removePid()
		return
	}

	process, err := os.FindProcess(pid)
	if err != nil {
		fmt.Printf("找不到进程: %v\n", err)
		removePid()
		return
	}

	err = process.Kill()
	if err != nil {
		fmt.Printf("停止失败: %v\n", err)
		return
	}

	removePid()
	fmt.Printf("服务已停止 (PID: %d)\n", pid)
}

func showStatus() {
	pid := getPid()
	if pid <= 0 {
		fmt.Println("服务状态: 未运行")
		return
	}

	if isProcessRunning(pid) {
		fmt.Printf("服务状态: 运行中\n")
		fmt.Printf("  PID:      %d\n", pid)
		fmt.Printf("  数据目录: %s\n", dataDir)
		fmt.Printf("  日志目录: %s\n", logDir)
	} else {
		fmt.Println("服务状态: 未运行 (PID文件已过期)")
		removePid()
	}
}

func resetData() {
	fmt.Print("确定要重置所有数据吗? 此操作不可恢复! [y/N]: ")
	var confirm string
	fmt.Scanln(&confirm)
	
	if strings.ToLower(confirm) != "y" {
		fmt.Println("已取消")
		return
	}

	// 删除数据文件
	if err := os.Remove(dataFile); err != nil && !os.IsNotExist(err) {
		fmt.Printf("删除数据文件失败: %v\n", err)
		return
	}

	// 清空日志目录
	os.RemoveAll(logDir)
	os.MkdirAll(logDir, 0755)

	fmt.Println("数据已重置")
}

func runServer() {
	// 初始化目录和日志
	initDirs()
	initLogger()

	// 保存当前进程PID
	savePid(os.Getpid())

	// 打印启动信息
	printBanner()
	appLogger.Printf("========================================")
	appLogger.Printf("  %s v%s", AppName, AppVersion)
	appLogger.Printf("========================================")
	appLogger.Printf("  PID:      %d", os.Getpid())
	appLogger.Printf("  端口:     %s", serverPort)
	appLogger.Printf("  数据目录: %s", dataDir)
	appLogger.Printf("  数据文件: %s", dataFile)
	appLogger.Printf("  日志目录: %s", logDir)
	appLogger.Printf("  访问地址: http://localhost:%s", serverPort)
	appLogger.Printf("========================================")

	// 加载API数据
	loadAPIs()

	mux := http.NewServeMux()

	// 嵌入的静态文件服务
	staticContent, _ := fs.Sub(staticFS, "static")
	mux.Handle("/static/", http.StripPrefix("/static/", http.FileServer(http.FS(staticContent))))

	// API管理接口
	mux.HandleFunc("/api/list", withLogging(listAPIsHandler))
	mux.HandleFunc("/api/save", withLogging(saveAPIHandler))
	mux.HandleFunc("/api/delete", withLogging(deleteAPIHandler))
	mux.HandleFunc("/api/logs", withLogging(getLogsHandler))
	mux.HandleFunc("/api/clear-logs", withLogging(clearLogsHandler))
	mux.HandleFunc("/api/reorder", withLogging(reorderAPIsHandler))

	// 所有其他请求走动态路由
	mux.HandleFunc("/", dynamicHandler)

	appLogger.Printf("服务启动成功，等待请求...")
	log.Fatal(http.ListenAndServe(fmt.Sprintf(":%s", serverPort), mux))
}

func printBanner() {
	fmt.Println(`
  __  __            _       _    ____ ___ 
 |  \/  | ___   ___| | __  / \  |  _ \_ _|
 | |\/| |/ _ \ / __| |/ / / _ \ | |_) | | 
 | |  | | (_) | (__|   < / ___ \|  __/| | 
 |_|  |_|\___/ \___|_|\_\_/   \_\_|  |___|
                                          `)
}

// 初始化目录
func initDirs() {
	os.MkdirAll(dataDir, 0755)
	os.MkdirAll(logDir, 0755)
}

// 初始化日志系统
func initLogger() {
	appLogFile := filepath.Join(logDir, fmt.Sprintf("app_%s.log", time.Now().Format("2006-01-02")))
	appFile, err := os.OpenFile(appLogFile, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		log.Fatalf("打开应用日志文件失败: %v", err)
	}
	appLogger = log.New(io.MultiWriter(os.Stdout, appFile), "[APP] ", log.LstdFlags)

	reqLogFile := filepath.Join(logDir, fmt.Sprintf("request_%s.log", time.Now().Format("2006-01-02")))
	reqFile, err := os.OpenFile(reqLogFile, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		log.Fatalf("打开请求日志文件失败: %v", err)
	}
	reqLogger = log.New(reqFile, "[REQ] ", log.LstdFlags)

	appLogger.Println("日志系统初始化完成")
}

func withLogging(handler http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()
		reqLogger.Printf("%s %s %s", r.Method, r.URL.Path, r.RemoteAddr)
		handler(w, r)
		reqLogger.Printf("%s %s 完成 耗时: %v", r.Method, r.URL.Path, time.Since(start))
	}
}

func loadAPIs() {
	data, err := os.ReadFile(dataFile)
	if err != nil {
		apis = []MockAPI{}
		appLogger.Printf("数据文件不存在，初始化空列表")
		return
	}
	if err := json.Unmarshal(data, &apis); err != nil {
		appLogger.Printf("解析数据文件失败: %v", err)
		apis = []MockAPI{}
		return
	}
	appLogger.Printf("加载了 %d 个API配置", len(apis))
}

func saveAPIsToFile() {
	data, _ := json.MarshalIndent(apis, "", "  ")
	if err := os.WriteFile(dataFile, data, 0644); err != nil {
		appLogger.Printf("保存数据文件失败: %v", err)
	}
}

func dynamicHandler(w http.ResponseWriter, r *http.Request) {
	path := r.URL.Path

	if path == "/" {
		content, err := staticFS.ReadFile("static/index.html")
		if err != nil {
			http.Error(w, "Page not found", http.StatusNotFound)
			return
		}
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.Write(content)
		return
	}

	apiMutex.RLock()
	var matchedAPI *MockAPI
	for i := range apis {
		if apis[i].URL == path {
			matchedAPI = &apis[i]
			break
		}
	}
	apiMutex.RUnlock()

	if matchedAPI == nil {
		http.NotFound(w, r)
		return
	}

	handleMockRequest(w, r, matchedAPI)
}

func handleMockRequest(w http.ResponseWriter, r *http.Request, api *MockAPI) {
	reqLogger.Printf("Mock请求: %s %s", r.Method, r.URL.Path)

	// 检查HTTP方法是否匹配
	if r.Method != api.Method {
		reqLogger.Printf("方法不匹配: 期望 %s, 实际 %s", api.Method, r.Method)
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusMethodNotAllowed)
		w.Write([]byte(fmt.Sprintf(`{"error": "Method not allowed. Expected %s, got %s"}`, api.Method, r.Method)))
		
		// 记录错误日志
		logEntry := LogEntry{
			Timestamp:  time.Now().Format("2006-01-02 15:04:05"),
			Method:     r.Method,
			URL:        r.URL.String(),
			Headers:    make(map[string]string),
			StatusCode: 405,
			Error:      fmt.Sprintf("Method not allowed. Expected %s, got %s", api.Method, r.Method),
		}
		
		for k, v := range r.Header {
			if len(v) > 0 {
				logEntry.Headers[k] = v[0]
			}
		}
		
		if r.Body != nil {
			body, _ := io.ReadAll(r.Body)
			logEntry.RequestBody = string(body)
		}
		
		apiMutex.Lock()
		for i := range apis {
			if apis[i].ID == api.ID {
				apis[i].Logs = append(apis[i].Logs, logEntry)
				if len(apis[i].Logs) > 100 {
					apis[i].Logs = apis[i].Logs[len(apis[i].Logs)-100:]
				}
				break
			}
		}
		saveAPIsToFile()
		apiMutex.Unlock()
		return
	}

	logEntry := LogEntry{
		Timestamp:  time.Now().Format("2006-01-02 15:04:05"),
		Method:     r.Method,
		URL:        r.URL.String(),
		Headers:    make(map[string]string),
		StatusCode: 200,
	}

	for k, v := range r.Header {
		if len(v) > 0 {
			logEntry.Headers[k] = v[0]
		}
	}

	if r.Body != nil {
		body, _ := io.ReadAll(r.Body)
		logEntry.RequestBody = string(body)
	}

	for k, v := range api.Headers {
		w.Header().Set(k, v)
	}

	if w.Header().Get("Content-Type") == "" {
		w.Header().Set("Content-Type", "application/json")
	}

	w.Write([]byte(api.ResponseBody))

	apiMutex.Lock()
	for i := range apis {
		if apis[i].ID == api.ID {
			apis[i].Logs = append(apis[i].Logs, logEntry)
			if len(apis[i].Logs) > 100 {
				apis[i].Logs = apis[i].Logs[len(apis[i].Logs)-100:]
			}
			break
		}
	}
	saveAPIsToFile()
	apiMutex.Unlock()
}

func listAPIsHandler(w http.ResponseWriter, r *http.Request) {
	apiMutex.RLock()
	defer apiMutex.RUnlock()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(apis)
}

func saveAPIHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var api MockAPI
	if err := json.NewDecoder(r.Body).Decode(&api); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if !strings.HasPrefix(api.URL, "/") {
		api.URL = "/" + api.URL
	}

	apiMutex.Lock()
	defer apiMutex.Unlock()

	now := time.Now().Format("2006-01-02 15:04:05")

	found := false
	for i := range apis {
		if apis[i].ID == api.ID {
			api.CreatedAt = apis[i].CreatedAt
			api.UpdatedAt = now
			api.Logs = apis[i].Logs
			apis[i] = api
			found = true
			appLogger.Printf("更新API: %s (%s)", api.Name, api.URL)
			break
		}
	}

	if !found {
		api.ID = fmt.Sprintf("%d", time.Now().UnixNano())
		api.CreatedAt = now
		api.UpdatedAt = now
		api.Logs = []LogEntry{}
		apis = append([]MockAPI{api}, apis...)
		appLogger.Printf("新增API: %s (%s)", api.Name, api.URL)
	}

	saveAPIsToFile()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]interface{}{"success": true, "api": api})
}

func deleteAPIHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		ID string `json:"id"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	apiMutex.Lock()
	defer apiMutex.Unlock()

	for i := range apis {
		if apis[i].ID == req.ID {
			appLogger.Printf("删除API: %s (%s)", apis[i].Name, apis[i].URL)
			apis = append(apis[:i], apis[i+1:]...)
			break
		}
	}

	saveAPIsToFile()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]bool{"success": true})
}

func getLogsHandler(w http.ResponseWriter, r *http.Request) {
	id := r.URL.Query().Get("id")

	apiMutex.RLock()
	defer apiMutex.RUnlock()

	for _, api := range apis {
		if api.ID == id {
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(api.Logs)
			return
		}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode([]LogEntry{})
}

func clearLogsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		ID string `json:"id"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	apiMutex.Lock()
	defer apiMutex.Unlock()

	for i := range apis {
		if apis[i].ID == req.ID {
			apis[i].Logs = []LogEntry{}
			appLogger.Printf("清空日志: %s", apis[i].Name)
			break
		}
	}

	saveAPIsToFile()
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]bool{"success": true})
}

func reorderAPIsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != "POST" {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req struct {
		IDs []string `json:"ids"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	apiMutex.Lock()
	defer apiMutex.Unlock()

	apiMap := make(map[string]MockAPI)
	for _, api := range apis {
		apiMap[api.ID] = api
	}

	newApis := make([]MockAPI, 0, len(apis))
	for _, id := range req.IDs {
		if api, ok := apiMap[id]; ok {
			newApis = append(newApis, api)
			delete(apiMap, id)
		}
	}

	for _, api := range apiMap {
		newApis = append(newApis, api)
	}

	apis = newApis
	saveAPIsToFile()
	appLogger.Println("API列表重新排序")

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]bool{"success": true})
}
