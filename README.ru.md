<div align="center">

![app preview](.meta/hotkeyhub-preview.png)

# ⌨️ HotkeyHub

### *Ваши горячие клавиши в одном окне*
**Незаменимая вещь для новичков в новом окружении**

<br>

[![Issues](https://img.shields.io/github/issues/meowrch/HotKeyHub?color=ffb29b&labelColor=1C2325&style=for-the-badge)](https://github.com/meowrch/HotKeyHub/issues)
[![Stars](https://img.shields.io/github/stars/meowrch/HotKeyHub?color=fab387&labelColor=1C2325&style=for-the-badge)](https://github.com/meowrch/HotKeyHub/stargazers)
[![License](https://img.shields.io/github/license/meowrch/HotKeyHub?color=FCA2AA&labelColor=1C2325&style=for-the-badge)](./LICENSE)

[![README RU](https://img.shields.io/badge/README-RU-blue?color=cba6f7&labelColor=cba6f7&style=for-the-badge)](./README.ru.md)
[![README ENG](https://img.shields.io/badge/README-ENG-blue?color=C9CBFF&labelColor=1C2325&style=for-the-badge)](./README.md)

[🚀 Быстрый старт](#quick-start) - [✨ Возможности](#features) - [🔧 Поддерживаемые форматы](#supported-formats) - [📚 FAQ](#faq)

</div>

***

## 🎯 Что такое HotkeyHub?
HotkeyHub - это приложение, отображающее сочетания клавиш.
Может помочь новичкам освоиться в новых дотфайлах. \
Просто добавьте сочетание клавиш `super + /` в свой WM и удобный Cheat Sheet будет всегда под рукой!


## <a name="features"></a>✨ Основные возможности

### 📋 Поддержка нескольких конфигураций

HotkeyHub [автоматически находит WM](#supported-formats) и парсит сочетания клавиш:
- **HyprLang** (`~/.config/hypr/hyprland.conf`)
- **HyprLua** (`~/.config/hypr/hyprland.lua`)
- **SXHKD** (`~/.config/sxhkd/sxhkdrc` или `~/.config/bspwm/sxhkdrc`)

Каждая конфигурация отображается в отдельной вкладке!

### 🔍 Умный поиск

- Поиск по **модификаторам** (Super, Ctrl, Alt)
- Поиск по **клавишам** (Return, Space)
- Поиск по **командам** (kitty, firefox, rofi)
- Поиск **в реальном времени**

## <a name="quick-start"></a>🚀 Быстрый старт

### Установка

#### Arch Linux (AUR)

```
# Скоро появится в AUR
yay -S hotkeyhub-bin
```

#### Сборка из исходников

```
# 1. Установите зависимости
sudo pacman -S rust gtk4 base-devel

# 2. Клонируйте репозиторий
git clone https://github.com/meowrch/HotkeyHub.git
cd HotkeyHub

# 3. Соберите
cargo build --release

# 4. Установите
sudo cp target/release/hotkeyhub /usr/bin/
```

### Первый запуск

```
# Запустите приложение
hotkeyhub

# Или запустите для конкретного конфига
hotkeyhub --hyprland ~/.config/hypr/hyprland.lua
hotkeyhub --sxhkd ~/.config/sxhkd/sxhkdrc
```

### Добавьте в ваш WM

**Hyprland (Hyprlang)** (`~/.config/hypr/hyprland.conf`):
```
bind = $mainMod, SLASH, exec, hotkeyhub  # Super + /
```

**Hyprland (Lua)** (`~/.config/hypr/hyprland.lua`):
```lua
hl.bind(mainMod .. " + SLASH", hl.dsp.exec_cmd("hotkeyhub --hyprland"), {description = "Hotkeys cheat sheet"})
```

**BSPWM (sxhkd)** (`~/.config/sxhkd/sxhkdrc`):
```
super + slash
    hotkeyhub
```

## <a name="shortcuts"></a>⌨️ Горячие клавиши

| Клавиши | Действие |
|---------|----------|
| **Ctrl + F** | Фокус на поле поиска |
| **Alt + 1-9** | Переключение между вкладками |
| **PgUp / PgDn** | Прокрутка списка |
| **Q** | Выход из приложения |

> [!TIP]
> При запуске курсор автоматически в поле поиска — сразу начинайте вводить!


## <a name="configuration"></a>Конфигурация 

Для HotkeyHub предусмотрена минимальная конфигурация.
Конфигурация выполняется в файле `~/.config/HotkeyHub/app.conf`
```
switch_description = правда
```

> [!NOTE]
> switch_description по умолчанию имеет значение false. Это переключает размещение описания и команды.

## <a name="themes"></a>🎨 Кастомизация тем

HotkeyHub поддерживает пользовательские темы через `~/.config/HotkeyHub/theme.conf`:

```
# Цвета темы (HEX формат)
background = #1e1e2e
background_alt = #313244
accent = #89b4fa
text = #cdd6f4
border = #45475a
```

> [!NOTE]
> Тема обновляется автоматически при изменении файла — перезапуск не нужен!

## <a name="supported-formats"></a>🔧 Поддерживаемые форматы

### Hyprland (Hyprlang)

```
# Простые бинды
bind = $mainMod, Return, exec, kitty

# С модификаторами
bind = $mainMod+Shift, Q, killactive

# Специальные клавиши
bind = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+

# Code клавиши
bind = $mainMod, code:60, exec, rofimoji  # code:60 = точка

# Мышь
bindm = $mainMod, mouse:272, movewindow   # ЛКМ
bindm = $mainMod, mouse:273, resizewindow # ПКМ
```

### Hyprland (Lua)

```lua
-- Simple binds

local term = "kitty"

hl.bind(mainMod .. " + Return", hl.dsp.exec_cmd(term), {description = "Open terminal"})

-- With modifiers
hl.bind("SUPER + SHIFT + Q", hl.dsp.window.kill(), {description = "Kill window"})

-- Special keys
hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"), {description = "Increase volume", repeating = true, locked = true})

-- Code keys
hl.bind(mainMod .. " + code:60", hl.dsp.exec_cmd("rofimoji"), {description = "Open emoji picker"})

-- Mouse
hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), {description = "Move window with mouse", mouse = true}) -- LMB
hl.bind(mainMod .. " + mouse:273", hl.dsp.window.resize(), {description = "Resize with mouse", mouse = true}) -- RMB
```

### SXHKD

```
# Простые бинды
super + Return
    kitty

# Множественные варианты
super + {_,shift + }{Left,Right,Up,Down}
    bspc node -{f,s} {west,east,north,south}

# XF86 клавиши
XF86Audio{RaiseVolume,LowerVolume,Mute}
    wpctl set-volume @DEFAULT_AUDIO_SINK@ {5%+,5%-,toggle}
```

## <a name="faq"></a>📚 FAQ

### Почему не отображаются некоторые бинды?
Убедитесь, что синтаксис в конфиге правильный. HotkeyHub пропускает строки с ошибками парсинга.

### Можно ли добавить поддержку i3/Sway?
Да! Откройте [Feature Request](https://github.com/meowrch/HotkeyHub/issues) — мы добавим поддержку.

### Как изменить шрифт?
Редактируйте CSS в коде или создайте issue с запросом на добавление настройки шрифта в `theme.conf`.

### Работает ли на Wayland?
Да, HotkeyHub использует GTK4, который полностью совместим с Wayland.

> [!TIP]
> **Не работает?** \
> [Откройте issue](https://github.com/meowrch/HotkeyHub/issues) с описанием проблемы.

## 🤝 Вклад в проект

Хотите улучшить HotkeyHub? Мы будем рады вашему вкладу!

1. Форкните репозиторий
2. Создайте ветку (`git checkout -b feature/AmazingFeature`)
3. Закоммитьте изменения (`git commit -m 'Add some AmazingFeature'`)
4. Запушьте в ветку (`git push origin feature/AmazingFeature`)
5. Откройте Pull Request

## 🗺️ Roadmap

- [ ] Поддержка i3/Sway
- [ ] Поддержка Niri
- [ ] Экспорт биндов в PDF/PNG
- [ ] Поддержка Awesome WM

## ☕ Поддержать проект

<div align="center">

**Нравится HotkeyHub?** Помогите проекту развиваться! 🚀

| 💎 Криптовалюта | 📬 Адрес |
|:---:|:---|
| **TON** | `UQB9qNTcAazAbFoeobeDPMML9MG73DUCAFTpVanQnLk3BHg3` |
| **Ethereum** | `0x56e8bf8Ec07b6F2d6aEdA7Bd8814DB5A72164b13` |
| **Bitcoin** | `bc1qt5urnw7esunf0v7e9az0jhatxrdd0smem98gdn` |
| **Tron** | `TBTZ5RRMfGQQ8Vpf8i5N8DZhNxSum2rzAs` |

<br>

*Каждое пожертвование мотивирует продолжать разработку! ❤️*

</div>

---

<div align="center">

**Сделано с ❤️ для Linux сообщества**

</div>