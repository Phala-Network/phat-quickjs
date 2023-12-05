import json
import xml.etree.ElementTree as ET
import xml.dom.minidom as minidom

def convert_to_xml(json_obj, parent=None):
    # Determine the tag and create the XML element
    tag = 'element'
    attributes = {
        'self': str(json_obj['shallow_size']),
        'retained': str(json_obj['retained_size']),
        'name': json_obj['name'],
    }
    
    if parent is None:
        # Create the root element if parent is None
        element = ET.Element(tag, attributes)
    else:
        # Otherwise, create a subelement
        element = ET.SubElement(parent, tag, attributes)

    # Recursively process children
    for child in json_obj.get('children', []):
        convert_to_xml(child, element)

    return element

# Function to convert the entire JSON to XML and return as a string
def json_to_xml_string(json_data):
    root_element = convert_to_xml(json_data)
    return ET.tostring(root_element, encoding='utf-8').decode('utf-8')

# Main script to read JSON file, convert and save as XML
def convert_json_to_xml(json_file_path, xml_file_path):
    with open(json_file_path, 'r') as file:
        json_data = json.load(file)

    xml_obj = convert_to_xml(json_data['items'][0])
    xml_pretty_str = minidom.parseString(ET.tostring(xml_obj)).toprettyxml()

    with open(xml_file_path, 'w') as file:
        file.write(xml_pretty_str)


# Example usage
json_file_path = 'domin.json'
xml_file_path = 'domin.xml'

convert_json_to_xml(json_file_path, xml_file_path)

