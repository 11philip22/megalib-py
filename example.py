#!/usr/bin/env python3
"""
Megalib Python Example - Demonstrates all features

Set environment variables before running:
    MEGA_EMAIL=user@example.com
    MEGA_PASSWORD=yourpassword

Optionally:
    MEGA_PROXY=http://proxy:8080
    MEGA_PUBLIC_FILE=https://mega.nz/file/...
    MEGA_PUBLIC_FOLDER=https://mega.nz/folder/...
"""

import asyncio
import os

import megalib


async def main():
    # ========================================
    # CONFIGURATION
    # ========================================
    email = os.environ.get("MEGA_EMAIL")
    password = os.environ.get("MEGA_PASSWORD")
    proxy = os.environ.get("MEGA_PROXY")
    public_file_url = os.environ.get("MEGA_PUBLIC_FILE")
    public_folder_url = os.environ.get("MEGA_PUBLIC_FOLDER")

    if not email or not password:
        print("❌ Please set MEGA_EMAIL and MEGA_PASSWORD environment variables")
        print("   Example (PowerShell):")
        print("   $env:MEGA_EMAIL='user@example.com'; $env:MEGA_PASSWORD='pass'; python example.py")
        return

    print("=" * 60)
    print("MEGALIB PYTHON BINDINGS - FEATURE DEMO")
    print("=" * 60)

    # ========================================
    # 1. LOGIN
    # ========================================
    print("\n📌 1. LOGIN")
    print("-" * 40)

    session = await megalib.MegaSession.login(email, password, proxy)
    print(f"✅ Logged in as: {email}")

    # ========================================
    # 2. SESSION INFO
    # ========================================
    print("\n📌 2. SESSION INFO")
    print("-" * 40)

    user_email = await session.get_email()
    user_name = await session.get_name()
    user_handle = await session.get_handle()

    print(f"   Email:  {user_email}")
    print(f"   Name:   {user_name}")
    print(f"   Handle: {user_handle}")

    # ========================================
    # 3. REFRESH FILESYSTEM
    # ========================================
    print("\n📌 3. REFRESH FILESYSTEM")
    print("-" * 40)

    await session.refresh()
    print("✅ Filesystem tree loaded")

    # ========================================
    # 4. QUOTA
    # ========================================
    print("\n📌 4. STORAGE QUOTA")
    print("-" * 40)

    total, used = await session.quota()
    used_mb = used / 1024 / 1024
    total_gb = total / 1024 / 1024 / 1024
    percent = (used / total) * 100 if total > 0 else 0

    print(f"   Used:  {used_mb:.2f} MB")
    print(f"   Total: {total_gb:.2f} GB")
    print(f"   Usage: {percent:.1f}%")

    # ========================================
    # 5. LIST ROOT
    # ========================================
    print("\n📌 5. LIST /Root")
    print("-" * 40)

    nodes = await session.list("/Root")
    for node in nodes[:10]:  # Show first 10
        icon = "📁" if node.is_folder else "📄"
        size = f"({node.size:,} bytes)" if node.is_file else ""
        print(f"   {icon} {node.name} {size}")
    if len(nodes) > 10:
        print(f"   ... and {len(nodes) - 10} more")

    # ========================================
    # 6. LIST RECURSIVE
    # ========================================
    print("\n📌 6. LIST RECURSIVE (first 5)")
    print("-" * 40)

    all_nodes = await session.list("/Root", recursive=True)
    for node in all_nodes[:5]:
        icon = "📁" if node.is_folder else "📄"
        print(f"   {icon} {node.name}")
    print(f"   Total: {len(all_nodes)} items")

    # ========================================
    # 7. STAT
    # ========================================
    print("\n📌 7. STAT /Root")
    print("-" * 40)

    root = await session.stat("/Root")
    if root:
        print(f"   Name:   {root.name}")
        print(f"   Handle: {root.handle}")
        print(f"   Type:   {'Folder' if root.is_folder else 'File'}")

    # ========================================
    # 8. CREATE FOLDER
    # ========================================
    print("\n📌 8. CREATE FOLDER")
    print("-" * 40)

    test_folder = "/Root/megalib_demo"
    try:
        await session.mkdir(test_folder)
        print(f"✅ Created: {test_folder}")
    except Exception as e:
        print(f"⚠️  {e} (folder may already exist)")

    # ========================================
    # 9. UPLOAD FILE
    # ========================================
    print("\n📌 9. UPLOAD FILE")
    print("-" * 40)

    test_file = "megalib_test.txt"
    with open(test_file, "w") as f:
        f.write("Hello from megalib Python bindings! 🚀\n")

    try:
        await session.upload(test_file, test_folder)
        print(f"✅ Uploaded: {test_file} -> {test_folder}")
    except Exception as e:
        print(f"❌ {e}")
    finally:
        os.remove(test_file)

    # ========================================
    # 10. LIST UPLOADED FILE
    # ========================================
    print("\n📌 10. LIST UPLOADED FILE")
    print("-" * 40)

    await session.refresh()  # Refresh to see new file
    folder_nodes = await session.list(test_folder)
    for node in folder_nodes:
        print(f"   📄 {node.name} ({node.size} bytes)")

    # ========================================
    # 11. DOWNLOAD FILE
    # ========================================
    print("\n📌 11. DOWNLOAD FILE")
    print("-" * 40)

    if folder_nodes:
        remote_path = f"{test_folder}/{folder_nodes[0].name}"
        local_path = "downloaded_test.txt"
        try:
            await session.download(remote_path, local_path)
            print(f"✅ Downloaded: {remote_path} -> {local_path}")
            with open(local_path) as f:
                print(f"   Content: {f.read().strip()}")
            os.remove(local_path)
        except Exception as e:
            print(f"❌ {e}")

    # ========================================
    # 12. RENAME FILE
    # ========================================
    print("\n📌 12. RENAME FILE")
    print("-" * 40)

    if folder_nodes:
        old_path = f"{test_folder}/{folder_nodes[0].name}"
        new_name = "renamed_test.txt"
        try:
            await session.rename(old_path, new_name)
            print(f"✅ Renamed: {folder_nodes[0].name} -> {new_name}")
        except Exception as e:
            print(f"❌ {e}")

    # ========================================
    # 13. EXPORT (CREATE PUBLIC LINK)
    # ========================================
    print("\n📌 13. EXPORT (PUBLIC LINK)")
    print("-" * 40)

    await session.refresh()
    folder_nodes = await session.list(test_folder)
    if folder_nodes:
        remote_path = f"{test_folder}/{folder_nodes[0].name}"
        try:
            url = await session.export(remote_path)
            print(f"✅ Public link: {url}")
        except Exception as e:
            print(f"❌ {e}")

    # ========================================
    # 14. DELETE FILE
    # ========================================
    print("\n📌 14. DELETE FILE")
    print("-" * 40)

    if folder_nodes:
        remote_path = f"{test_folder}/{folder_nodes[0].name}"
        try:
            await session.rm(remote_path)
            print(f"✅ Deleted: {remote_path}")
        except Exception as e:
            print(f"❌ {e}")

    # ========================================
    # 15. DELETE FOLDER
    # ========================================
    print("\n📌 15. DELETE FOLDER")
    print("-" * 40)

    try:
        await session.rm(test_folder)
        print(f"✅ Deleted: {test_folder}")
    except Exception as e:
        print(f"❌ {e}")

    # ========================================
    # 16. SESSION CONFIGURATION
    # ========================================
    print("\n📌 16. SESSION CONFIGURATION")
    print("-" * 40)

    await session.set_workers(4)
    print("   Workers set to: 4")

    await session.set_resume(True)
    print("   Resume: enabled")

    await session.enable_previews(True)
    print("   Previews: enabled")

    # ========================================
    # 17. SAVE SESSION
    # ========================================
    print("\n📌 17. SAVE SESSION")
    print("-" * 40)

    session_file = ".mega_session.json"
    await session.save(session_file)
    print(f"✅ Session saved to: {session_file}")

    # ========================================
    # 18. LOAD SESSION
    # ========================================
    print("\n📌 18. LOAD SESSION")
    print("-" * 40)

    loaded = await megalib.MegaSession.load(session_file)
    if loaded:
        print(f"✅ Session loaded from: {session_file}")
        email = await loaded.get_email()
        print(f"   Email: {email}")
    os.remove(session_file)

    # ========================================
    # 19. LIST CONTACTS
    # ========================================
    print("\n📌 19. LIST CONTACTS")
    print("-" * 40)

    contacts = await session.list_contacts()
    if contacts:
        for contact in contacts[:5]:
            print(f"   👤 {contact.name}")
    else:
        print("   (no contacts)")

    # ========================================
    # 20. PUBLIC FILE (NO LOGIN REQUIRED)
    # ========================================
    print("\n📌 20. PUBLIC FILE INFO")
    print("-" * 40)

    if public_file_url:
        try:
            info = await megalib.get_public_file_info(public_file_url)
            print(f"   Name:   {info.name}")
            print(f"   Size:   {info.size:,} bytes")
            print(f"   Handle: {info.handle}")
        except Exception as e:
            print(f"❌ {e}")
    else:
        print("   (set MEGA_PUBLIC_FILE to test)")

    # ========================================
    # 21. PUBLIC FOLDER (NO LOGIN REQUIRED)
    # ========================================
    print("\n📌 21. PUBLIC FOLDER BROWSING")
    print("-" * 40)

    if public_folder_url:
        try:
            folder = await megalib.open_folder(public_folder_url)
            nodes = await folder.list("/")
            for node in nodes[:5]:
                icon = "📁" if node.is_folder else "📄"
                print(f"   {icon} {node.name}")
        except Exception as e:
            print(f"❌ {e}")
    else:
        print("   (set MEGA_PUBLIC_FOLDER to test)")

    # ========================================
    # DONE
    # ========================================
    print("\n" + "=" * 60)
    print("✅ ALL FEATURES DEMONSTRATED SUCCESSFULLY")
    print("=" * 60)


if __name__ == "__main__":
    asyncio.run(main())
