### RDP服务（TCP+UDP）
```yaml
- name: "RDP"
  listen_port: 99
  protocols: ["tcp", "udp"]
  buffer_size: 32768
  targets:
    - "192.168.5.12:3389"
    - "hz.ipto.top:57111"
  dynamic_update:
    enabled: true
    check_interval: 15  # RDP连接敏感，频繁检查
```

**RDP UDP优化说明**：
- **TCP协议**：提供可靠的连接和完整的RDP功能
- **UDP协议**：优化连接性能，减少延迟，提升用户体验
- **应用场景**：远程桌面连接，支持高分辨率显示和多媒体传输
