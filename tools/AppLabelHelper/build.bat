@echo off
chcp 65001 >nul
:: AppLabelHelper DEX 编译脚本
:: 依赖：ANDROID_HOME 环境变量指向 Android SDK 根目录

setlocal

if "%ANDROID_HOME%"=="" (
    echo [ERROR] 请先设置 ANDROID_HOME 环境变量，例如:
    echo   set ANDROID_HOME=C:\Users\xxx\AppData\Local\Android\Sdk
    exit /b 1
)

:: 查找最新 build-tools 版本
for /f "delims=" %%i in ('dir /b /o-n "%ANDROID_HOME%\build-tools"') do (
    set BUILD_TOOLS=%%i
    goto :found
)

:found
set D8=%ANDROID_HOME%\build-tools\%BUILD_TOOLS%\d8.bat
set ANDROID_JAR=%ANDROID_HOME%\platforms\android-34\android.jar

if not exist "%ANDROID_JAR%" (
    :: 回退到找任意可用 android.jar
    for /f "delims=" %%j in ('dir /b /o-n "%ANDROID_HOME%\platforms\android-*\android.jar" 2^>nul') do (
        set ANDROID_JAR=%%j
        goto :jar_found
    )
    echo [ERROR] 未找到 android.jar，请安装 Android SDK Platform
    exit /b 1
)

:jar_found
echo [INFO] 使用 Build-Tools: %BUILD_TOOLS%
echo [INFO] 使用 android.jar: %ANDROID_JAR%

:: 第一步：javac 编译 Java -> class
javac -cp "%ANDROID_JAR%" -source 8 -target 8 Main.java
if errorlevel 1 (
    echo [ERROR] javac 编译失败
    exit /b 1
)

:: 第二步：d8 转换 class -> dex
call "%D8%" Main.class --output . --min-api 24
if errorlevel 1 (
    echo [ERROR] d8 编译 DEX 失败
    exit /b 1
)

:: 复制到资源目录
set OUT=..\..\bin\app-tools
if not exist "%OUT%" mkdir "%OUT%"
copy /y classes.dex "%OUT%\AppLabelHelper.dex"
endlocal
pause