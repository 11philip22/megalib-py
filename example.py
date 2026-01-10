import asyncio
import os
import megalib

async def main():
    # 1. Get credentials from environment
    email = os.getenv("MEGA_EMAIL")
    password = os.getenv("MEGA_PASSWORD")

    if not email or not password:
        print("Please set MEGA_EMAIL and MEGA_PASSWORD environment variables.")
        print("Example: $env:MEGA_EMAIL='user@example.com'; $env:MEGA_PASSWORD='password'; python example.py")
        return

    # 2. Login
    print(f"Logging in as {email}...")
    try:
        session = await megalib.MegaSession.login(email, password)
    except ValueError as e:
        print(f"Login failed: {e}")
        return

    print(f"Logged in! Fetching filesystem...")
    
    # 3. Refresh session to load file system
    await session.refresh()
    
    # 4. Get Quota
    total, used = await session.quota()
    print(f"Storage: {used / 1024 / 1024:.2f} MB / {total / 1024 / 1024 / 1024:.2f} GB")

    # 5. List Root
    print("\nListing /Root:")
    nodes = await session.list("/Root")
    for node in nodes:
        icon = "📁" if node.is_folder else "📄"
        print(f"  {icon} {node.name} ({node.size} bytes)")

    # 6. Create a test folder
    folder_name = "megalib_python_demo"
    print(f"\nCreating folder: /Root/{folder_name}")
    try:
        await session.mkdir(f"/Root/{folder_name}")
        print("✅ Folder created.")
    except RuntimeError: # Might fail if exists
        print("⚠️  Folder might already exist.")

    # 7. Upload a file
    filename = "test_upload.txt"
    content = "Hello from megalib Python bindings! 🚀"
    
    print(f"\nCreating local file: {filename}")
    with open(filename, "w", encoding="utf-8") as f:
        f.write(content)
    
    print(f"Uploading to /Root/{folder_name}...")
    try:
        await session.upload(filename, f"/Root/{folder_name}")
        print("✅ Upload complete.")
    except Exception as e:
        print(f"❌ Upload failed: {e}")

    # 8. List the new folder
    print(f"\nListing /Root/{folder_name}:")
    nodes = await session.list(f"/Root/{folder_name}")
    for node in nodes:
        print(f"  📄 {node.name}")

    # 9. Export the uploaded file
    remote_path = f"/Root/{folder_name}/{filename}"
    print(f"\nExporting {remote_path}...")
    try:
        url = await session.export(remote_path)
        print(f"🔗 Public Link: {url}")
        
        # 10. Get Public File Info
        print("\nFetching info from public link...")
        info = await megalib.get_public_file_info(url)
        print(f"  File: {info.name}")
        print(f"  Size: {info.size} bytes")
        
    except Exception as e:
        print(f"❌ Export/Info failed: {e}")

    # Clean up local file
    if os.path.exists(filename):
        os.remove(filename)

if __name__ == "__main__":
    asyncio.run(main())
