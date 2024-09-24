import yt_dlp
import argparse
import os

def download_video(url, output_path='./', ffmpeg_path=None, download_audio_only=False):
    ydl_opts = {
        'format': 'bestaudio/best' if download_audio_only else 'bestvideo+bestaudio/best',
        'outtmpl': os.path.join(output_path, '%(title)s.%(ext)s'),
        'user_agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
        'nocheckcertificate': True,
        'ignoreerrors': False,
        'no_warnings': False,
        'verbose': True,
    }
    
    if download_audio_only:
        ydl_opts['postprocessors'] = [{
            'key': 'FFmpegExtractAudio',
            'preferredcodec': 'mp3',
            'preferredquality': '192',
        }]
    else:
        ydl_opts['postprocessors'] = [{
            'key': 'FFmpegVideoConvertor',
            'preferedformat': 'mp4',
        }]
    
    if ffmpeg_path:
        ydl_opts['ffmpeg_location'] = ffmpeg_path
    
    try:
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            ydl.download([url])
    except yt_dlp.utils.DownloadError as e:
        print(f"An error occurred: {str(e)}")
        print("Please try updating yt-dlp or report the issue to the yt-dlp GitHub repository.")

def main():
    parser = argparse.ArgumentParser(description='Download YouTube videos or audio.')
    parser.add_argument('url', help='YouTube video URL')
    parser.add_argument('-o', '--output', default='./', help='Output path')
    parser.add_argument('-f', '--ffmpeg', help='FFmpeg path')
    parser.add_argument('-a', '--audio-only', action='store_true', help='Download audio only')
    
    args = parser.parse_args()
    
    download_video(args.url, args.output, args.ffmpeg, args.audio_only)

if __name__ == '__main__':
    main()