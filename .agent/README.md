# .agent — AI Agent Context Directory

Thư mục này chứa context và skill sets được dùng bởi AI coding agent cho project Vaultwarden.

## Cấu Trúc

```
.agent/
└── skills/
    └── rust-expert/         ← Bộ skill chuyên gia Rust
        ├── metadata.json         ← Index & tags
        ├── persona.md            ← Danh tính & phong cách làm việc
        ├── core_skills.md        ← Kỹ năng Rust cốt lõi
        ├── vaultwarden_stack.md  ← Stack kỹ thuật của project
        ├── coding_standards.md  ← Coding conventions & lint rules
        ├── security_expertise.md ← Chuyên môn bảo mật
        └── patterns_and_recipes.md ← Code templates sẵn dùng
```

## Cách Dùng

Khi làm việc với AI agent trên project này, agent sẽ đọc các file trong `.agent/skills/`
để hiểu ngữ cảnh và áp dụng đúng conventions, patterns của project.

> **Lưu ý**: Thư mục này đã được thêm vào `.gitignore` nếu bạn không muốn commit lên repo.
> Hoặc commit để share context với team.
