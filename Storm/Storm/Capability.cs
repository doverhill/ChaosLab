using System.Collections.Generic;
using System.Linq;

namespace Storm {
    internal class Capability {
        public string Namespace;
        public string Operation;

        public enum CapabilityType {
            None,
            Name,
            Numeric,
            NumericRange
        }

        public CapabilityType Type;
        public string ResourceName;
        public ulong NumericValue;
        public ulong NumericEndValue;

        // allowed capability types:
        // Namespace.Operation
        // Namespace.Operation:ResourceName
        // Namespace.Operation:#number
        // Namespace.Operation:#number-number
        internal static ErrorOr<Capability> Parse(string capabilityString) {
            if (string.IsNullOrEmpty(capabilityString)) return ErrorOr<Capability>.Error(ErrorCode.Malformed);

            var parts = capabilityString.Split('.');
            if (parts.Length != 2) return ErrorOr<Capability>.Error(ErrorCode.Malformed);

            var operationParts = parts[1].Split(":");

            if (!IsValidNamespace(parts[0]) || !IsValidOperation(operationParts[0])) return ErrorOr<Capability>.Error(ErrorCode.Malformed);

            var parameters = operationParts.Take(1).ToArray();

            var type = CapabilityType.None;
            string resourceName = null;
            ulong numericValue = 0;
            ulong numericEndValue = 0;

            if (parameters.Length == 1) {
                var value = parameters[0];
                if (value.Length < 1) return ErrorOr<Capability>.Error(ErrorCode.Malformed);

                // parameters might be ResourceName or #number
                if (value[0] == '#') {
                    if (!ulong.TryParse(value[1..], out var longValue)) return ErrorOr<Capability>.Error(ErrorCode.Malformed);
                    type = CapabilityType.Numeric;
                    numericValue = longValue;
                }
                else {
                    type = CapabilityType.Name;
                    resourceName = parameters[0];
                }
            }
            else if (parameters.Length == 2) {
                var value1 = parameters[0];
                var value2 = parameters[1];
                if (value1.Length < 2 || value1[0] != '#' || value2.Length < 2 || value2[0] != '#') return ErrorOr<Capability>.Error(ErrorCode.Malformed);
                if (!ulong.TryParse(value1[1..], out var longValue1) || !ulong.TryParse(value2[1..], out var longValue2)) return ErrorOr<Capability>.Error(ErrorCode.Malformed);
                if (longValue2 <= longValue1) return ErrorOr<Capability>.Error(ErrorCode.Malformed);
                type = CapabilityType.NumericRange;
                numericValue = longValue1;
                numericEndValue = longValue2;
            }
            else if (parameters.Length > 2) {
                return ErrorOr<Capability>.Error(ErrorCode.Malformed);
            }

            return ErrorOr<Capability>.Ok(new Capability {
                Namespace = parts[0],
                Operation = operationParts[0],
                Type = type,
                ResourceName = resourceName,
                NumericValue = numericValue,
                NumericEndValue = numericEndValue
            });
        }

        private static bool IsValidNamespace(string namespaceString) {
            return IsValidPascalCase(namespaceString);
        }

        private static bool IsValidOperation(string operationString) {
            return IsValidPascalCase(operationString);
        }

        private static bool IsValidPascalCase(string value) {
            bool first = true;
            bool lastUppercase = true;

            var isUppercase = (char c) => {
                return c >= 'A' && c <= 'Z';
            };

            var isLowercase = (char c) => {
                return c >= 'a' && c <= 'z';
            };

            var isValidCharacter = (char c) => {
                return isLowercase(c) || isUppercase(c);
            };

            foreach (var character in value) {
                if (!isValidCharacter(character)) return false;

                if (first) {
                    if (!isUppercase(character)) return false;
                    first = false;
                }
                else {
                    var uppercase = isUppercase(character);
                    if (uppercase && lastUppercase) return false;
                    lastUppercase = uppercase;
                }
            }

            return true;
        }

        public static bool IsSubset(List<Capability> parentCapabilities, List<Capability> childCapabilities) {
            foreach (var child in childCapabilities) {
                // each child capability must be present in parent capabilities
                var found = parentCapabilities.Where(pc => pc.Namespace == child.Namespace && pc.Operation == child.Operation && pc.Type == child.Type).ToList();
                if (!found.Any()) return false;

                var anyMatching = false;
                foreach (var parent in found) {
                    switch (child.Type) {
                        case CapabilityType.None:
                            anyMatching = true;
                            break;

                        case CapabilityType.Name:
                            if (child.ResourceName == parent.ResourceName) anyMatching = true;
                            break;

                        case CapabilityType.Numeric:
                            if (child.NumericValue == parent.NumericValue) anyMatching = true;
                            break;

                        case CapabilityType.NumericRange:
                            if (child.NumericValue >= parent.NumericValue && child.NumericEndValue <= parent.NumericEndValue) anyMatching = true;
                            break;
                    }
                }

                if (!anyMatching) return false;
            }
            return true;
        }
    }
}