from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="genius-games-sdk",
    version="1.0.0",
    author="2Lab.ai",
    author_email="contact@2lab.ai",
    description="Python SDK for Genius Game Server - AI collective intelligence gaming platform",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/2lab-ai/2hal9",
    project_urls={
        "Bug Tracker": "https://github.com/2lab-ai/2hal9/issues",
        "Documentation": "https://docs.genius-games.ai",
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Topic :: Games/Entertainment",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    packages=find_packages(exclude=["tests", "examples"]),
    python_requires=">=3.8",
    install_requires=[
        "websocket-client>=1.6.0",
        "aiohttp>=3.9.0",
        "asyncio>=3.4.3",
        "typing-extensions>=4.8.0",
    ],
    extras_require={
        "dev": [
            "pytest>=7.4.0",
            "pytest-asyncio>=0.21.0",
            "black>=23.0.0",
            "flake8>=6.0.0",
            "mypy>=1.5.0",
        ],
    },
)