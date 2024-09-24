import tkinter as tk
from tkinter import messagebox
import psutil
import socket
import threading
import select

class ProxyGUI:
    def __init__(self, master):
        self.master = master
        master.title("Connection Proxy GUI")

        self.label = tk.Label(master, text="Target PID: 9088")
        self.label.pack()

        self.ip_label = tk.Label(master, text="Proxy Server IP:")
        self.ip_label.pack()
        self.ip_entry = tk.Entry(master)
        self.ip_entry.pack()

        self.port_label = tk.Label(master, text="Proxy Server Port:")
        self.port_label.pack()
        self.port_entry = tk.Entry(master)
        self.port_entry.pack()

        self.secret_label = tk.Label(master, text="Secret Code:")
        self.secret_label.pack()
        self.secret_entry = tk.Entry(master, show="*")
        self.secret_entry.pack()

        self.proxy_button = tk.Button(master, text="Start Proxying", command=self.toggle_proxy)
        self.proxy_button.pack()

        self.status_label = tk.Label(master, text="Status: Not Proxying")
        self.status_label.pack()

        self.is_proxying = False
        self.proxy_socket = None
        self.proxy_thread = None

    def toggle_proxy(self):
        if self.is_proxying:
            self.stop_proxy()
        else:
            self.start_proxy()

    def start_proxy(self):
        if not psutil.pid_exists(9088):
            messagebox.showerror("Error", "Process with PID 9088 not found!")
            return

        ip = self.ip_entry.get()
        port = self.port_entry.get()
        secret = self.secret_entry.get()

        if not ip or not port or not secret:
            messagebox.showerror("Error", "Please fill in all fields!")
            return

        try:
            port = int(port)
        except ValueError:
            messagebox.showerror("Error", "Invalid port number!")
            return

        try:
            self.proxy_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.proxy_socket.connect((ip, port))

            # Perform handshake
            self.proxy_socket.send(secret.encode())
            response = self.proxy_socket.recv(1024).decode()

            if response != "OK":
                messagebox.showerror("Error", "Handshake failed!")
                self.proxy_socket.close()
                return

            self.is_proxying = True
            self.proxy_button.config(text="Stop Proxying")
            self.status_label.config(text="Status: Proxying")

            # Start proxy thread
            self.proxy_thread = threading.Thread(target=self.proxy_connections)
            self.proxy_thread.start()

        except Exception as e:
            messagebox.showerror("Error", f"Failed to start proxy: {str(e)}")
            if self.proxy_socket:
                self.proxy_socket.close()

    def stop_proxy(self):
        if self.proxy_socket:
            self.proxy_socket.close()
        self.is_proxying = False
        self.proxy_button.config(text="Start Proxying")
        self.status_label.config(text="Status: Not Proxying")
        # The proxy_thread will terminate when the socket is closed

    def proxy_connections(self):
        try:
            while self.is_proxying:
                # This is a simple example that forwards stdin to the proxy server
                # and prints responses. You would need to modify this to intercept
                # actual network traffic from the target process.
                rlist, _, _ = select.select([self.proxy_socket, socket.stdin], [], [], 1)
                for ready_socket in rlist:
                    if ready_socket == self.proxy_socket:
                        data = self.proxy_socket.recv(4096)
                        if not data:
                            raise Exception("Connection closed by proxy server")
                        print(f"Received from proxy: {data.decode()}")
                    elif ready_socket == socket.stdin:
                        data = input()
                        self.proxy_socket.send(data.encode())
        except Exception as e:
            print(f"Proxy error: {str(e)}")
            self.stop_proxy()

root = tk.Tk()
gui = ProxyGUI(root)
root.mainloop()