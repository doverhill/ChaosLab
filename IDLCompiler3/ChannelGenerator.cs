using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace IDLCompiler
{
    internal static class ChannelGenerator
    {
        public static void GenerateChannel(SourceGenerator source, IDL idl)
        {
            var protocolName = CasedString.FromSnake(idl.Protocol.Name);
            var channelName = $"{protocolName.ToPascal()}Channel";

            source.AddLine("use std::sync::atomic::{AtomicBool, Ordering};");
            source.AddBlank();

            var versionBlock = source.AddBlock("struct ProtocolVersion");
            versionBlock.AddLine("major: u16,");
            versionBlock.AddLine("minor: u16,");
            versionBlock.AddLine("patch: u16,");
            versionBlock.AddLine("is_preview: bool,");
            versionBlock.AddLine("preview_version: u16,");

            source.AddBlank();

            var channelHeader = source.AddBlock("struct ChannelHeader");
            channelHeader.AddLine("lock: AtomicBool,");
            channelHeader.AddLine("channel_magic: u64,");
            channelHeader.AddLine("protocol_name: [u8; 32],");
            channelHeader.AddLine("protocol_version: ProtocolVersion,");
            channelHeader.AddLine("first_message_offset: usize,");
            channelHeader.AddLine("last_message_offset: usize,");
            channelHeader.AddLine("number_of_messages: usize,");
            channelHeader.AddLine("is_writing: bool,");

            source.AddBlank();

            var channeHeaderlImpl = source.AddBlock("impl ChannelHeader");
            channeHeaderlImpl.AddLine("pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'C' as u8, 'H' as u8, 'A' as u8, 'N' as u8, 'N' as u8, 'E' as u8, 'L' as u8]);");

            source.AddBlank();

            var messageHeader = source.AddBlock("pub struct ChannelMessageHeader");
            messageHeader.AddLine("message_magic: u64,");
            messageHeader.AddLine("message_id: usize,");
            messageHeader.AddLine("message_length: usize,");
            messageHeader.AddLine("previous_message_offset: usize,");
            messageHeader.AddLine("next_message_offset: usize,");
            messageHeader.AddLine("replace_pending: bool,");

            source.AddBlank();

            var messageImpl = source.AddBlock("impl ChannelMessageHeader");
            messageImpl.AddLine("pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8]);");

            source.AddBlank();

            var lockBlock = source.AddBlock("struct ChannelLock");
            lockBlock.AddLine("channel_header: *const ChannelHeader,");

            source.AddBlank();

            var lockImpl = source.AddBlock("impl ChannelLock");
            var getFunction = lockImpl.AddBlock($"pub unsafe fn get(channel: &{channelName}) -> Self");
            getFunction.AddLine("let channel_header = channel.channel_address as *const ChannelHeader;");
            getFunction.AddLine("while (*channel_header).lock.swap(true, Ordering::Acquire) {}");
            var getReturn = getFunction.AddBlock("Self");
            getReturn.AddLine("channel_header: channel_header");

            source.AddBlank();

            var dropImpl = source.AddBlock("impl Drop for ChannelLock");
            var dropFunction = dropImpl.AddBlock("fn drop(&mut self)");
            var dropUnsafe = dropFunction.AddBlock("unsafe");
            dropUnsafe.AddLine("(*self.channel_header).lock.swap(false, Ordering::Release);");

            source.AddBlank();

            var channel = source.AddBlock($"pub struct {channelName}");
            channel.AddLine("channel_address: *mut u8,");

            source.AddBlank();

            var channelImpl = source.AddBlock($"impl {channelName}");

            var newFunction = channelImpl.AddBlock("pub unsafe fn new(channel_address: *mut u8, is_server: bool) -> Self");
            var ifBlock = newFunction.AddBlock("if !is_server");
            ifBlock.AddLine("let channel_header = channel_address as *mut ChannelHeader;");
            ifBlock.AddLine("(*channel_header).lock.store(false, Ordering::Relaxed);");
            ifBlock.AddLine("(*channel_header).channel_magic = ChannelHeader::MAGIC;");
            ifBlock.AddLine($"(*channel_header).protocol_name[0] = {idl.Protocol.Name.Length};");
            for (var i = 0; i < idl.Protocol.Name.Length; i++)
            {
                ifBlock.AddLine($"(*channel_header).protocol_name[{i + 1}] = '{idl.Protocol.Name[i]}' as u8;");
            }
            versionBlock = ifBlock.AddBlock("(*channel_header).protocol_version = ProtocolVersion");
            versionBlock.AddLine($"major: {idl.Protocol.Version},");
            versionBlock.AddLine("minor: 0,");
            versionBlock.AddLine("patch: 0,");
            versionBlock.AddLine("is_preview: false,");
            versionBlock.AddLine("preview_version: 0,");
            versionBlock.SemiColonAfter = true;
            ifBlock.AddLine("(*channel_header).first_message_offset = 0;");
            ifBlock.AddLine("(*channel_header).last_message_offset = 0;");
            ifBlock.AddLine("(*channel_header).number_of_messages = 0;");
            ifBlock.AddLine("(*channel_header).is_writing = false;");

            var returnBlock = newFunction.AddBlock(channelName);
            returnBlock.AddLine("channel_address: channel_address,");

            channelImpl.AddBlank();

            var compatibleBlock = channelImpl.AddBlock("pub unsafe fn check_compatible(&self) -> bool");
            compatibleBlock.AddLine("let channel_header = self.channel_address as *mut ChannelHeader;");

            var checkString = "(*channel_header).channel_magic == ChannelHeader::MAGIC";
            checkString += $" && (*channel_header).protocol_version.major == {idl.Protocol.Version}";
            checkString += $" && (*channel_header).protocol_name[0] == {idl.Protocol.Name.Length}";
            for (var i = 0; i < idl.Protocol.Name.Length; i++)
            {
                checkString += $" && (*channel_header).protocol_name[{i + 1}] == '{idl.Protocol.Name[i]}' as u8";
            }
            compatibleBlock.AddLine(checkString);

            channelImpl.AddBlank();

            var prepareBlock = channelImpl.AddBlock("pub unsafe fn prepare_message(&self, message_id: usize, replace_pending: bool) -> *mut u8");
            prepareBlock.AddLine("let channel_header = self.channel_address as *mut ChannelHeader;");
            prepareBlock.AddLine("let lock = ChannelLock::get(self);");
            prepareBlock.AddLine("#[cfg(debug)]");
            prepareBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");
            prepareBlock.AddLine("assert!(!(*channel_header).is_writing);");
            prepareBlock.AddLine("let mut message: *mut ChannelMessageHeader;");
            ifBlock = prepareBlock.AddBlock("if (*channel_header).number_of_messages == 0");
            ifBlock.AddLine("(*channel_header).first_message_offset = mem::size_of::<ChannelHeader>();");
            ifBlock.AddLine("(*channel_header).last_message_offset = mem::size_of::<ChannelHeader>();");
            ifBlock.AddLine("message = self.channel_address.offset(mem::size_of::<ChannelHeader>() as isize) as *mut ChannelMessageHeader;");
            ifBlock.AddLine("(*message).previous_message_offset = 0;");
            var elseBlock = prepareBlock.AddBlock("else");
            elseBlock.AddLine("let last_message_offset = (*channel_header).last_message_offset;");
            elseBlock.AddLine("let last_message = self.channel_address.offset(last_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*last_message).next_message_offset = (*channel_header).last_message_offset + (*last_message).message_length;");
            elseBlock.AddLine("message = self.channel_address.offset((*last_message).next_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*message).previous_message_offset = last_message_offset;");

            prepareBlock.AddLine("(*channel_header).is_writing = true;");
            //prepareBlock.AddLine("#[cfg(debug)]");
            prepareBlock.AddLine("(*message).message_magic = ChannelMessageHeader::MAGIC;");
            prepareBlock.AddLine("(*message).message_id = message_id;");
            prepareBlock.AddLine("(*message).replace_pending = replace_pending;");
            prepareBlock.AddLine("(*message).message_length = 0;");
            prepareBlock.AddLine("(*message).next_message_offset = 0;");
            prepareBlock.AddLine("message as *mut u8");

            channelImpl.AddBlank();

            var commitBlock = channelImpl.AddBlock("pub unsafe fn commit_message(&self, message_payload_size: usize)");
            commitBlock.AddLine("let channel_header = self.channel_address as *mut ChannelHeader;");
            commitBlock.AddLine("let lock = ChannelLock::get(self);");
            commitBlock.AddLine("let last_message = self.channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;");
            commitBlock.AddLine("#[cfg(debug)]");
            commitBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");
            commitBlock.AddLine("assert!((*channel_header).is_writing);");
            commitBlock.AddLine("#[cfg(debug)]");
            commitBlock.AddLine("assert!((*last_message).message_magic == ChannelMessageHeader::MAGIC);");
            commitBlock.AddLine("(*channel_header).is_writing = false;");
            commitBlock.AddLine("(*channel_header).number_of_messages = (*channel_header).number_of_messages + 1;");
            commitBlock.AddLine("(*last_message).message_length = mem::size_of::<ChannelMessageHeader>() + message_payload_size;");

            channelImpl.AddBlank();

            var findBlock = channelImpl.AddBlock("pub unsafe fn find_message(&self) -> Option<*mut ChannelMessageHeader>");
            findBlock.AddLine("let channel_header = self.channel_address as *mut ChannelHeader;");
            findBlock.AddLine("let lock = ChannelLock::get(self);");
            findBlock.AddLine("#[cfg(debug)]");
            findBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");

            ifBlock = findBlock.AddBlock("if (*channel_header).number_of_messages == 0");
            ifBlock.AddLine("None");

            elseBlock = findBlock.AddBlock("else");
            elseBlock.AddLine("let first_message = self.channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("#[cfg(debug)]");
            elseBlock.AddLine("assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);");
            ifBlock = elseBlock.AddBlock("if !(*first_message).replace_pending");
            ifBlock.AddLine("Some(first_message)");
            elseBlock = elseBlock.AddBlock("else");
            elseBlock.AddLine("let mut last_of_kind = first_message;");
            elseBlock.AddLine("let iter = first_message;");
            var whileBlock = elseBlock.AddBlock("while (*iter).next_message_offset != 0");
            whileBlock.AddLine("let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;");
            ifBlock = whileBlock.AddBlock("if (*iter).message_id == (*first_message).message_id");
            ifBlock.AddLine("last_of_kind = iter;");
            elseBlock.AddLine("let iter = first_message;");
            whileBlock = elseBlock.AddBlock("while (*iter).next_message_offset != 0");
            ifBlock = whileBlock.AddBlock("if (*iter).message_id == (*first_message).message_id && iter != last_of_kind");
            ifBlock.AddLine("assert!((*channel_header).number_of_messages > 1);");
            ifBlock.AddLine("self.unlink_message(iter, true);");
            whileBlock.AddLine("let iter = self.channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("Some(last_of_kind)");

            channelImpl.AddBlank();

            var unlinkBlock = channelImpl.AddBlock("pub unsafe fn unlink_message(&self, message: *mut ChannelMessageHeader, lock_held: bool)");
            unlinkBlock.AddLine("let channel_header = self.channel_address as *mut ChannelHeader;");
            unlinkBlock.AddLine("let lock = if lock_held { None } else { Some(ChannelLock::get(self)) };");
            unlinkBlock.AddLine("#[cfg(debug)]");
            unlinkBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");
            unlinkBlock.AddLine("#[cfg(debug)]");
            unlinkBlock.AddLine("assert!((*message).message_magic == ChannelMessageHeader::MAGIC);");

            ifBlock = unlinkBlock.AddBlock("if (*message).previous_message_offset == 0");
            ifBlock.AddLine("(*channel_header).first_message_offset = (*message).next_message_offset;");
            elseBlock = unlinkBlock.AddBlock("else");
            elseBlock.AddLine("let previous_message = self.channel_address.offset((*message).previous_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*previous_message).next_message_offset = (*message).next_message_offset;");

            ifBlock = unlinkBlock.AddBlock("if (*message).next_message_offset == 0");
            ifBlock.AddLine("(*channel_header).last_message_offset = (*message).previous_message_offset;");
            elseBlock = unlinkBlock.AddBlock("else");
            elseBlock.AddLine("let next_message = self.channel_address.offset((*message).next_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*next_message).previous_message_offset = (*message).previous_message_offset;");

            unlinkBlock.AddLine("(*channel_header).number_of_messages = (*channel_header).number_of_messages - 1;");
        }
    }
}
