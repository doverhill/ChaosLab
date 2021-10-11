
namespace Storm
{
    partial class StormUI
    {
        /// <summary>
        ///  Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        ///  Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        ///  Required method for Designer support - do not modify
        ///  the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            this.KernelConsole = new System.Windows.Forms.RichTextBox();
            this.SuspendLayout();
            // 
            // KernelConsole
            // 
            this.KernelConsole.BackColor = System.Drawing.Color.Black;
            this.KernelConsole.Dock = System.Windows.Forms.DockStyle.Fill;
            this.KernelConsole.ForeColor = System.Drawing.Color.White;
            this.KernelConsole.Location = new System.Drawing.Point(0, 0);
            this.KernelConsole.Name = "KernelConsole";
            this.KernelConsole.Size = new System.Drawing.Size(800, 450);
            this.KernelConsole.TabIndex = 0;
            this.KernelConsole.Text = "";
            // 
            // StormUI
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(7F, 15F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Font;
            this.ClientSize = new System.Drawing.Size(800, 450);
            this.Controls.Add(this.KernelConsole);
            this.Name = "StormUI";
            this.Text = "Storm";
            this.Load += new System.EventHandler(this.Form1_Load);
            this.ResumeLayout(false);

        }

        #endregion

        private System.Windows.Forms.RichTextBox KernelConsole;
    }
}

