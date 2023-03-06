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

            source.AddLine("use core::sync::atomic::{AtomicBool, Ordering};");
            source.AddLine("use core::ops::Deref;");
            source.AddLine("use core::marker::PhantomData;");
            source.AddBlank();

            var fromChannelStruct = source.AddBlock("pub struct FromChannel<T>");
            fromChannelStruct.AddLine("channel_address: *mut u8,");
            fromChannelStruct.AddLine("message_address: *mut ChannelMessageHeader,");
            fromChannelStruct.AddLine("phantom: PhantomData<T>,");
            source.AddBlank();

            var fromChannelImpl = source.AddBlock("impl<T> FromChannel<T>");
            var newBlock = fromChannelImpl.AddBlock($"pub fn new (channel_address: *mut u8, message_address: *mut ChannelMessageHeader) -> Self");
            var selfBlock = newBlock.AddBlock("Self");
            selfBlock.AddLine("channel_address: channel_address,");
            selfBlock.AddLine("message_address: message_address,");
            selfBlock.AddLine("phantom: PhantomData,");
            source.AddBlank();

            var derefImpl = source.AddBlock("impl<T> Deref for FromChannel<T>");
            derefImpl.AddLine("type Target = T;");
            var derefFunc = derefImpl.AddBlock("fn deref(&self) -> &T");
            derefFunc.AddLine("unsafe { (ChannelMessageHeader::get_payload_address(self.message_address) as *const T).as_ref().unwrap() }");
            source.AddBlank();

            var dropImpl = source.AddBlock("impl<T> Drop for FromChannel<T>");
            var dropFunc = dropImpl.AddBlock("fn drop(&mut self)");
            var unsafeBlock = dropFunc.AddBlock("unsafe");
            unsafeBlock.AddLine("let channel_header = self.channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine("""let lock = ChannelLock::get("from_channel_drop", self.channel_address);""");
            unsafeBlock.AddLine("assert!((*self.message_address).pending_unlink);");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*self.channel_address).channel_magic == ChannelHeader::MAGIC);");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*self.message_address).message_magic == ChannelMessageHeader::MAGIC);");
            var ifBlock = unsafeBlock.AddBlock("if (*self.message_address).previous_message_offset == 0");
            ifBlock.AddLine("(*channel_header).first_message_offset = (*self.message_address).next_message_offset;");
            var elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let previous_message = self.channel_address.offset((*self.message_address).previous_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*previous_message).next_message_offset = (*self.message_address).next_message_offset;");
            ifBlock = unsafeBlock.AddBlock("if (*self.message_address).next_message_offset == 0");
            ifBlock.AddLine("(*channel_header).last_message_offset = (*self.message_address).previous_message_offset;");
            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let next_message = self.channel_address.offset((*self.message_address).next_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*next_message).previous_message_offset = (*self.message_address).previous_message_offset;");

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
            //channelHeader.AddLine("number_of_messages: usize,");
            channelHeader.AddLine("is_writing: bool,");

            source.AddBlank();

            var channeHeaderlImpl = source.AddBlock("impl ChannelHeader");
            channeHeaderlImpl.AddLine("pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'C' as u8, 'H' as u8, 'A' as u8, 'N' as u8, 'N' as u8, 'E' as u8, 'L' as u8]);");

            source.AddBlank();

            var messageHeader = source.AddBlock("pub struct ChannelMessageHeader");
            messageHeader.AddLine("message_magic: u64,");
            messageHeader.AddLine("pub message_id: u64,");
            messageHeader.AddLine("pub call_id: u64,");
            messageHeader.AddLine("message_length: usize,");
            messageHeader.AddLine("previous_message_offset: usize,");
            messageHeader.AddLine("next_message_offset: usize,");
            messageHeader.AddLine("replace_pending: bool,");
            messageHeader.AddLine("is_writing: bool,");
            messageHeader.AddLine("pending_unlink: bool,");

            source.AddBlank();

            var messageImpl = source.AddBlock("impl ChannelMessageHeader");
            messageImpl.AddLine("pub const MAGIC: u64 = u64::from_be_bytes(['C' as u8, 'M' as u8, 'E' as u8, 'S' as u8, 'S' as u8, 'A' as u8, 'G' as u8, 'E' as u8]);");
            messageImpl.AddBlank();
            var getAddressBlock = messageImpl.AddBlock("pub fn get_payload_address(message: *mut ChannelMessageHeader) -> *mut u8");
            getAddressBlock.AddLine("unsafe { message.offset(mem::size_of::<ChannelMessageHeader>() as isize) as *mut u8 }");

            source.AddBlank();

            var lockBlock = source.AddBlock("struct ChannelLock");
            lockBlock.AddLine("name: String,");
            lockBlock.AddLine("channel_header: *const ChannelHeader,");

            source.AddBlank();

            var lockImpl = source.AddBlock("impl ChannelLock");
            var getFunction = lockImpl.AddBlock($"pub unsafe fn get(name: &str, channel_address: *mut u8) -> Self");
            getFunction.AddLine("let channel_header = channel_address as *const ChannelHeader;");
            //getFunction.AddLine("""println!("LOCK: getting for {}", name);""");
            getFunction.AddLine("while (*channel_header).lock.swap(true, Ordering::Acquire) {}");
            //getFunction.AddLine("""println!("LOCK: got for {}", name);""");
            var getReturn = getFunction.AddBlock("Self");
            getReturn.AddLine("name: name.to_string(),");
            getReturn.AddLine("channel_header: channel_header");

            source.AddBlank();

            dropImpl = source.AddBlock("impl Drop for ChannelLock");
            var dropFunction = dropImpl.AddBlock("fn drop(&mut self)");
            //dropFunction.AddLine("""println!("LOCK: releasing for {}", self.name);""");
            var dropUnsafe = dropFunction.AddBlock("unsafe");
            dropUnsafe.AddLine("(*self.channel_header).lock.store(false, Ordering::Relaxed);");

            source.AddBlank();

            var channel = source.AddBlock($"pub struct {channelName}");
            channel.AddLine("pub rx_channel_address: *mut u8,");
            channel.AddLine("tx_channel_address: *mut u8,");
            channel.AddLine("call_id: u64,");

            source.AddBlank();

            var channelImpl = source.AddBlock($"impl {channelName}");

            var newFunctionBlock = channelImpl.AddBlock("pub fn new(channel_address_0: *mut u8, channel_address_1: *mut u8, is_server: bool) -> Self");
            unsafeBlock = newFunctionBlock.AddBlock("unsafe");
            ifBlock = unsafeBlock.AddBlock("if is_server");
            var returnBlock = ifBlock.AddBlock(channelName);
            returnBlock.AddLine("rx_channel_address: channel_address_0,");
            returnBlock.AddLine("tx_channel_address: channel_address_1,");
            returnBlock.AddLine("call_id: 1,");
            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("Self::initialize(channel_address_0);");
            elseBlock.AddLine("Self::initialize(channel_address_1);");
            returnBlock = elseBlock.AddBlock(channelName);
            returnBlock.AddLine("rx_channel_address: channel_address_1,");
            returnBlock.AddLine("tx_channel_address: channel_address_0,");
            returnBlock.AddLine("call_id: 1,");

            channelImpl.AddBlank();

            var initializeBlock = channelImpl.AddBlock("unsafe fn initialize(channel_address: *mut u8)");
            initializeBlock.AddLine("let channel_header = channel_address as *mut ChannelHeader;");
            initializeBlock.AddLine("(*channel_header).lock = AtomicBool::new(false);");
            initializeBlock.AddLine("(*channel_header).channel_magic = ChannelHeader::MAGIC;");
            initializeBlock.AddLine($"(*channel_header).protocol_name[0] = {idl.Protocol.Name.Length};");
            for (var i = 0; i < idl.Protocol.Name.Length; i++) {
                initializeBlock.AddLine($"(*channel_header).protocol_name[{i + 1}] = '{idl.Protocol.Name[i]}' as u8;");
            }
            versionBlock = initializeBlock.AddBlock("(*channel_header).protocol_version = ProtocolVersion");
            versionBlock.AddLine($"major: {idl.Protocol.Version},");
            versionBlock.AddLine("minor: 0,");
            versionBlock.AddLine("patch: 0,");
            versionBlock.AddLine("is_preview: false,");
            versionBlock.AddLine("preview_version: 0,");
            versionBlock.SemiColonAfter = true;
            initializeBlock.AddLine("(*channel_header).first_message_offset = 0;");
            initializeBlock.AddLine("(*channel_header).last_message_offset = 0;");
            //initializeBlock.AddLine("(*channel_header).number_of_messages = 0;");
            initializeBlock.AddLine("(*channel_header).is_writing = false;");

            channelImpl.AddBlank();

            var compatibleFunctionBlock = channelImpl.AddBlock("pub fn check_compatible(&self) -> bool");
            unsafeBlock = compatibleFunctionBlock.AddBlock("unsafe");
            var checkString = "(*channel_header).channel_magic == ChannelHeader::MAGIC";
            checkString += $" && (*channel_header).protocol_version.major == {idl.Protocol.Version}";
            checkString += $" && (*channel_header).protocol_name[0] == {idl.Protocol.Name.Length}";
            for (var i = 0; i < idl.Protocol.Name.Length; i++)
            {
                checkString += $" && (*channel_header).protocol_name[{i + 1}] == '{idl.Protocol.Name[i]}' as u8";
            }
            unsafeBlock.AddLine("let channel_header = self.rx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine($"let rx_compatible = {checkString};");
            unsafeBlock.AddLine("let channel_header = self.tx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine($"let tx_compatible = {checkString};");
            unsafeBlock.AddLine("rx_compatible && tx_compatible");
            channelImpl.AddBlank();

            GenerateDump(channelImpl, "rx");
            GenerateDump(channelImpl, "tx");

            GenerateCount(channelImpl, "rx");
            GenerateCount(channelImpl, "tx");

            var prepareFunctionBlock = channelImpl.AddBlock("pub fn prepare_message(&mut self, message_id: u64, replace_pending: bool) -> (u64, *mut ChannelMessageHeader)");
            //prepareFunctionBlock.AddLine("self.dump_tx(\"prepare_message BEFORE\");");
            unsafeBlock = prepareFunctionBlock.AddBlock("unsafe");
            unsafeBlock.AddLine("let channel_header = self.tx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine("""let lock = ChannelLock::get("prepare_message", self.tx_channel_address);""");
            unsafeBlock.AddLine("let call_id = self.call_id;");
            unsafeBlock.AddLine("self.call_id += 1;");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");
            unsafeBlock.AddLine("assert!(!(*channel_header).is_writing);");
            unsafeBlock.AddLine("let mut message: *mut ChannelMessageHeader;");
            ifBlock = unsafeBlock.AddBlock("if (*channel_header).first_message_offset == 0");
            ifBlock.AddLine("(*channel_header).first_message_offset = mem::size_of::<ChannelHeader>();");
            ifBlock.AddLine("(*channel_header).last_message_offset = mem::size_of::<ChannelHeader>();");
            ifBlock.AddLine("message = self.tx_channel_address.offset(mem::size_of::<ChannelHeader>() as isize) as *mut ChannelMessageHeader;");
            ifBlock.AddLine("(*message).previous_message_offset = 0;");
            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let last_message_offset = (*channel_header).last_message_offset;");
            elseBlock.AddLine("let last_message = self.tx_channel_address.offset(last_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("let new_last_message_offset = last_message_offset + (*last_message).message_length;");
            elseBlock.AddLine("(*last_message).next_message_offset = new_last_message_offset;");
            elseBlock.AddLine("(*channel_header).last_message_offset = new_last_message_offset;");
            elseBlock.AddLine("message = self.tx_channel_address.offset(new_last_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*message).previous_message_offset = last_message_offset;");

            unsafeBlock.AddLine("(*channel_header).is_writing = true;");
            //unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("(*message).message_magic = ChannelMessageHeader::MAGIC;");
            unsafeBlock.AddLine("(*message).message_id = message_id;");
            unsafeBlock.AddLine("(*message).call_id = call_id;");
            unsafeBlock.AddLine("(*message).replace_pending = replace_pending;");
            unsafeBlock.AddLine("(*message).message_length = 0;");
            unsafeBlock.AddLine("(*message).next_message_offset = 0;");
            unsafeBlock.AddLine("(*message).is_writing = true;");
            unsafeBlock.AddLine("(*message).pending_unlink = false;");
            //unsafeBlock.AddLine("self.dump_tx(\"prepare_message AFTER\");");
            unsafeBlock.AddLine("(call_id, message)");

            channelImpl.AddBlank();

            var commitFunctionBlock = channelImpl.AddBlock("pub fn commit_message(&self, message_payload_size: usize)");
            //commitFunctionBlock.AddLine("self.dump_tx(\"commit_message BEFORE\");");
            unsafeBlock = commitFunctionBlock.AddBlock("unsafe");
            unsafeBlock.AddLine("let channel_header = self.tx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine("""let lock = ChannelLock::get("commit_message", self.tx_channel_address);""");
            unsafeBlock.AddLine("let last_message = self.tx_channel_address.offset((*channel_header).last_message_offset as isize) as *mut ChannelMessageHeader;");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");
            unsafeBlock.AddLine("assert!((*channel_header).is_writing);");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*last_message).message_magic == ChannelMessageHeader::MAGIC);");
            unsafeBlock.AddLine("(*channel_header).is_writing = false;");
            //unsafeBlock.AddLine("(*channel_header).number_of_messages = (*channel_header).number_of_messages + 1;");
            unsafeBlock.AddLine("(*last_message).message_length = mem::size_of::<ChannelMessageHeader>() + message_payload_size;");
            unsafeBlock.AddLine("(*last_message).is_writing = false;");
            //unsafeBlock.AddLine("self.dump_tx(\"commit_message AFTER\");");

            channelImpl.AddBlank();

            var findFunctionBlock = channelImpl.AddBlock("pub fn find_specific_message(&self, call_id: u64) -> Option<*mut ChannelMessageHeader>");
            unsafeBlock = findFunctionBlock.AddBlock("unsafe");
            unsafeBlock.AddLine("let channel_header = self.rx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine("""let lock = ChannelLock::get("find_specific_message", self.rx_channel_address);""");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");

            ifBlock = unsafeBlock.AddBlock("if (*channel_header).first_message_offset == 0");
            ifBlock.AddLine("None");

            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let first_message = self.rx_channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("#[cfg(debug)]");
            elseBlock.AddLine("assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);");
            elseBlock.AddLine("let mut iter = first_message;");
            var whileBlock = elseBlock.AddBlock("while (*iter).call_id != call_id && (*iter).next_message_offset != 0");
            whileBlock.AddLine("iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;");
            ifBlock = elseBlock.AddBlock("if (*iter).call_id == call_id && !(*iter).is_writing && !(*iter).pending_unlink");
            //ifBlock.AddLine("assert!(!(*iter).pending_unlink);");
            ifBlock.AddLine("(*iter).pending_unlink = true;");
            ifBlock.AddLine("Some(iter)");
            elseBlock = elseBlock.AddBlock("else");
            elseBlock.AddLine("None");

            channelImpl.AddBlank();


            findFunctionBlock = channelImpl.AddBlock("pub fn find_message(&self) -> Option<*mut ChannelMessageHeader>");
            unsafeBlock = findFunctionBlock.AddBlock("unsafe");
            unsafeBlock.AddLine("let channel_header = self.rx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine("""let lock = ChannelLock::get("find_message", self.rx_channel_address);""");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");

            ifBlock = unsafeBlock.AddBlock("if (*channel_header).first_message_offset == 0");
            ifBlock.AddLine("None");

            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let mut iter = self.rx_channel_address.offset((*channel_header).first_message_offset as isize) as *mut ChannelMessageHeader;");
            whileBlock = elseBlock.AddBlock("while (*iter).next_message_offset != 0 && (*iter).pending_unlink");
            whileBlock.AddLine("iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;");

            elseBlock.AddLine("let first_message = iter;");
            elseBlock.AddLine("#[cfg(debug)]");
            elseBlock.AddLine("assert!((*first_message).message_magic == ChannelMessageHeader::MAGIC);");
            ifBlock = elseBlock.AddBlock("if !(*first_message).replace_pending");
            var innerIfBlock = ifBlock.AddBlock("if (*first_message).is_writing || (*first_message).pending_unlink");
            innerIfBlock.AddLine("None");
            var innerElseBlock = ifBlock.AddBlock("else");
            innerElseBlock.AddLine("(*first_message).pending_unlink = true;");
            innerElseBlock.AddLine("Some(first_message)");
            elseBlock = elseBlock.AddBlock("else");
            elseBlock.AddLine("let mut last_of_kind = first_message;");
            elseBlock.AddLine("let mut iter = first_message;");
            whileBlock = elseBlock.AddBlock("while (*iter).next_message_offset != 0 && !(*iter).is_writing");
            whileBlock.AddLine("iter = self.rx_channel_address.offset((*iter).next_message_offset as isize) as *mut ChannelMessageHeader;");
            ifBlock = whileBlock.AddBlock("if (*iter).message_id == (*first_message).message_id && !(*iter).pending_unlink");
            ifBlock.AddLine("last_of_kind = iter;");
            elseBlock.AddLine("let mut iter = first_message;");
            whileBlock = elseBlock.AddBlock("while (*iter).next_message_offset != 0 && iter != last_of_kind && !(*iter).is_writing");
            whileBlock.AddLine("let next_message_offset = (*iter).next_message_offset;");
            ifBlock = whileBlock.AddBlock("if (*iter).message_id == (*first_message).message_id && !(*iter).pending_unlink");
            //ifBlock.AddLine("assert!((*channel_header).number_of_messages > 1);");
            ifBlock.AddLine("self.unlink_message(iter, true);");
            whileBlock.AddLine("iter = self.rx_channel_address.offset(next_message_offset as isize) as *mut ChannelMessageHeader;");
            ifBlock = elseBlock.AddBlock("if (*last_of_kind).is_writing || (*last_of_kind).pending_unlink");
            ifBlock.AddLine("None");
            elseBlock = elseBlock.AddBlock("else");
            elseBlock.AddLine("(*last_of_kind).pending_unlink = true;");
            elseBlock.AddLine("Some(last_of_kind)");

            channelImpl.AddBlank();

            var unlinkFunctionBlock = channelImpl.AddBlock("pub fn unlink_message(&self, message: *mut ChannelMessageHeader, lock_held: bool)");
            unsafeBlock = unlinkFunctionBlock.AddBlock("unsafe");
            unsafeBlock.AddLine("let channel_header = self.rx_channel_address as *mut ChannelHeader;");
            unsafeBlock.AddLine("""let lock = if lock_held { None } else { Some(ChannelLock::get("unlink_message", self.rx_channel_address)) };""");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*channel_header).channel_magic == ChannelHeader::MAGIC);");
            unsafeBlock.AddLine("#[cfg(debug)]");
            unsafeBlock.AddLine("assert!((*message).message_magic == ChannelMessageHeader::MAGIC);");

            ifBlock = unsafeBlock.AddBlock("if (*message).previous_message_offset == 0");
            ifBlock.AddLine("(*channel_header).first_message_offset = (*message).next_message_offset;");
            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let previous_message = self.rx_channel_address.offset((*message).previous_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*previous_message).next_message_offset = (*message).next_message_offset;");

            ifBlock = unsafeBlock.AddBlock("if (*message).next_message_offset == 0");
            ifBlock.AddLine("(*channel_header).last_message_offset = (*message).previous_message_offset;");
            elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let next_message = self.rx_channel_address.offset((*message).next_message_offset as isize) as *mut ChannelMessageHeader;");
            elseBlock.AddLine("(*next_message).previous_message_offset = (*message).previous_message_offset;");

            //unsafeBlock.AddLine("(*channel_header).number_of_messages = (*channel_header).number_of_messages - 1;");
        }

        private static void GenerateDump(SourceGenerator.SourceBlock block, string channel)
        {
            var dumpBlock = block.AddBlock($"pub fn dump_{channel}(&self, text: &str)");
            var unsafeBlock = dumpBlock.AddBlock("unsafe");
            unsafeBlock.AddLine($"let channel_address = self.{channel}_channel_address;");
            unsafeBlock.AddLine($"println!(\"DUMPING CHANNEL {channel} ({{}}) {{:p}}\", text, channel_address);");
            unsafeBlock.AddLine("let channel_header = channel_address as *mut ChannelHeader;");
            //unsafeBlock.AddLine($"let lock = ChannelLock::get(\"dump_{channel}\", channel_address);");
            var ifBlock = unsafeBlock.AddBlock("if (*channel_header).first_message_offset == 0");
            ifBlock.AddLine("println!(\"  EMPTY\");");
            var elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let mut index = (*channel_header).first_message_offset;");
            elseBlock.AddLine("let mut iter = channel_address.offset((*channel_header).first_message_offset as isize) as *const ChannelMessageHeader;");
            var whileBlock = elseBlock.AddBlock("'messages: loop");
            whileBlock.AddLine("println!(\"  {}: prev: {}, next: {}, size: {}, message_id: {}, is_writing: {}, pending_unlink: {}\", index, (*iter).previous_message_offset, (*iter).next_message_offset, (*iter).message_length, (*iter).message_id, (*iter).is_writing, (*iter).pending_unlink);");
            ifBlock = whileBlock.AddBlock("if (*iter).next_message_offset == 0");
            ifBlock.AddLine("break 'messages;");
            whileBlock.AddLine("index = (*iter).next_message_offset;");
            whileBlock.AddLine("iter = channel_address.offset((*iter).next_message_offset as isize) as *const ChannelMessageHeader;");

            block.AddBlank();
        }

        private static void GenerateCount(SourceGenerator.SourceBlock block, string channel) {
            var dumpBlock = block.AddBlock($"pub fn message_count_{channel}(&self) -> usize");
            var unsafeBlock = dumpBlock.AddBlock("unsafe");
            unsafeBlock.AddLine($"let channel_address = self.{channel}_channel_address;");
            unsafeBlock.AddLine("let channel_header = channel_address as *mut ChannelHeader;");
            var ifBlock = unsafeBlock.AddBlock("if (*channel_header).first_message_offset == 0");
            ifBlock.AddLine("0");
            var elseBlock = unsafeBlock.AddBlock("else");
            elseBlock.AddLine("let mut count = 1;");
            elseBlock.AddLine("let mut iter = channel_address.offset((*channel_header).first_message_offset as isize) as *const ChannelMessageHeader;");
            var whileBlock = elseBlock.AddBlock("'messages: loop");
            ifBlock = whileBlock.AddBlock("if (*iter).next_message_offset == 0");
            ifBlock.AddLine("break 'messages;");
            whileBlock.AddLine("count += 1;");
            whileBlock.AddLine("iter = channel_address.offset((*iter).next_message_offset as isize) as *const ChannelMessageHeader;");
            elseBlock.AddLine("count");
            block.AddBlank();
        }
    }
}
